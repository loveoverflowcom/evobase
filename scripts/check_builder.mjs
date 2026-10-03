#!/usr/bin/env node
// Drive the mounted Rust/Leptos Builder, using synthetic fixtures and local browser storage.
// Dependencies may live outside the repository. Numeric assertions inspect exact Rust JSON bytes.
// PLAYWRIGHT_MODULE_PATH=/path/to/playwright/index.mjs \
// AXE_MODULE_PATH=/path/to/@axe-core/playwright/dist/index.mjs \
// BUILDER_URL=http://127.0.0.1:4173 EVIDENCE_DIR=/approved/run/path node scripts/check_builder.mjs
// SCENARIO_FILTER=history selects a focused subset; selecting zero scenarios fails.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const baseURL = process.env.BUILDER_URL ?? 'http://127.0.0.1:4173';
const evidenceDir = process.env.EVIDENCE_DIR;
const scenarioFilter = process.env.SCENARIO_FILTER ? new RegExp(process.env.SCENARIO_FILTER, 'i') : null;
if (!evidenceDir) throw new Error('EVIDENCE_DIR must name the approved synthetic QA artifact directory.');
const moduleURL = (value, fallback) => value ? pathToFileURL(path.resolve(value)).href : fallback;
const { chromium } = await import(moduleURL(process.env.PLAYWRIGHT_MODULE_PATH, 'playwright'));
const axeModule = await import(moduleURL(process.env.AXE_MODULE_PATH, '@axe-core/playwright'));
const AxeBuilder = typeof axeModule.default === 'function' ? axeModule.default : axeModule.default.default;
const storageKey = 'evobase.builder.local.v1';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir(evidenceDir, { recursive: true });
const browser = await chromium.launch({ headless: true });
const results = [];
const captures = [];
const resourceDigests = new Map();
const resourceReads = new Map();
const resourceErrors = [];
let skipped = 0;
const source = {
  head: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
  status: execFileSync('git', ['status', '--short'], { cwd: root, encoding: 'utf8' }).trim(),
  files: {},
};
for (const file of ['scripts/check_builder.mjs', 'apps/evobase-builder/src/lib.rs', 'apps/evobase-builder/src/relations.rs', 'apps/evobase-builder/src/policy.rs', 'apps/evobase-builder/builder.css', 'apps/evobase-builder/tokens.css', 'apps/evobase-builder/index.html']) {
  try {
    source.files[file] = digest(await readFile(path.join(root, file)));
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
}

const byId = (page, id) => page.getByTestId(id);
async function valueIs(page, id, value) {
  await page.waitForFunction(({ id, value }) => document.querySelector(`[data-testid="${id}"]`)?.value === value, { id, value });
}
async function textIncludes(page, id, text) {
  await page.waitForFunction(({ id, text }) => document.querySelector(`[data-testid="${id}"]`)?.textContent.includes(text), { id, text });
}
async function focused(page, id) {
  await page.waitForFunction(id => document.activeElement?.getAttribute('data-testid') === id, id);
}
async function attributeIs(page, id, name, value) {
  await page.waitForFunction(({ id, name, value }) => document.querySelector(`[data-testid="${id}"]`)?.getAttribute(name) === value, { id, name, value });
}
async function open(page) {
  await page.goto(baseURL, { waitUntil: 'networkidle' });
  await byId(page, 'app-name').waitFor({ state: 'visible' });
}
async function english(page) {
  if (await page.locator('html').getAttribute('lang') !== 'en') await byId(page, 'locale').click();
  await attributeIs(page, 'locale', 'data-testid', 'locale');
  assert.equal(await page.locator('html').getAttribute('lang'), 'en');
}
async function select(page, table) {
  await byId(page, `table-${table}`).click();
  await attributeIs(page, `table-${table}`, 'aria-pressed', 'true');
}
async function save(page) {
  await byId(page, 'save').click();
  await page.waitForFunction(() => document.querySelector('[data-testid="draft-status"]')?.getAttribute('data-state') === 'saved' && !document.querySelector('[data-testid="save"]')?.disabled);
}
async function rawStorage(page) {
  return page.evaluate(key => localStorage.getItem(key), storageKey);
}
async function envelope(page) {
  const raw = await rawStorage(page);
  assert.notEqual(raw, null, 'a storage receipt must exist');
  // Only the envelope is parsed. Inner definition/record bytes remain exact strings.
  const receipt = JSON.parse(raw);
  assert.equal(receipt.format, 1);
  assert.match(receipt.revision, /^(0|[1-9][0-9]*)$/);
  assert.equal(typeof receipt.definition_json, 'string');
  assert.equal(typeof receipt.records_json, 'string');
  return receipt;
}
function stringField(receipt, id, field) {
  // This helper is limited to explicitly text-typed fixtures. Numeric cases use raw bytes.
  const rows = JSON.parse(receipt.records_json);
  const row = rows.find(row => row.id === id);
  assert.ok(row, `record ${id} exists`);
  assert.equal(row.values[field].type, 'text');
  assert.equal(typeof row.values[field].value, 'string');
  return row.values[field].value;
}
async function rows(page, expected) {
  await page.waitForFunction(expected => document.querySelectorAll('[data-testid="typed-table"] tbody tr').length === expected, expected);
}
async function captureScenario(page, name, state) {
  const screenshot = path.join(evidenceDir, `${name}.png`);
  await page.screenshot({ path: screenshot, fullPage: true });
  const bytes = await readFile(screenshot);
  captures.push({ name, state, ...page.viewportSize(), locale: await page.locator('html').getAttribute('lang'), theme: await page.locator('html').getAttribute('data-theme') ?? 'light', origin: 'browser-runtime', path: screenshot, bytes: bytes.length, sha256: digest(bytes), inspected: false });
}
async function relationLine(page, id, expected) {
  const cells = await byId(page, 'relation-lines').locator(`tr[data-record-id="${id}"]`).locator('th,td').allTextContents();
  assert.deepEqual(cells, [id, ...expected]);
}
async function addOrderRule(page) {
  await english(page);
  await select(page, 'tbl_orders');
  await byId(page, 'policy-add-source').click();
  await textIncludes(page, 'policy-source', 'rule_order_owner');
}

async function test(name, run) {
  if (scenarioFilter && !scenarioFilter.test(name)) { skipped += 1; return; }
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, locale: 'vi-VN', reducedMotion: 'reduce' });
  const page = await context.newPage();
  const consoleErrors = [];
  const responseTasks = [];
  page.setDefaultTimeout(8000);
  page.on('pageerror', error => consoleErrors.push(`pageerror: ${error.message}`));
  page.on('console', event => { if (event.type() === 'error') consoleErrors.push(event.text()); });
  page.on('response', response => {
    if (/\.(wasm|js|css)(?:\?|$)/.test(response.url())) {
      if (!resourceReads.has(response.url())) resourceReads.set(response.url(), (async () => {
        let bytes;
        let boundary = 'browser response bytes';
        try {
          bytes = await response.body();
        } catch {
          // Debug WASM may exceed Chromium's inspector cache. Attribute that digest
          // to the served read-back rather than claiming inspector response bytes.
          const reread = await fetch(response.url());
          assert.equal(reread.status, 200);
          bytes = Buffer.from(await reread.arrayBuffer());
          boundary = 'served resource read-back';
        }
        resourceDigests.set(response.url(), { sha256: digest(bytes), bytes: bytes.length, boundary });
      })().catch(error => resourceErrors.push({ url: response.url(), message: error.message })));
      responseTasks.push(resourceReads.get(response.url()));
    }
  });
  const started = performance.now();
  try {
    await open(page);
    await run(page, context);
    await Promise.all(responseTasks);
    assert.deepEqual(consoleErrors, [], 'mounted app has no browser runtime errors');
    results.push({ name, passed: true, milliseconds: Math.round(performance.now() - started) });
    console.log(`PASS ${name}`);
  } catch (error) {
    const screenshot = path.join(evidenceDir, `failure-${results.length + 1}.png`);
    await page.screenshot({ path: screenshot, fullPage: true }).catch(() => {});
    const domPath = path.join(evidenceDir, `failure-${results.length + 1}.html`);
    await writeFile(domPath, await page.content()).catch(() => {});
    results.push({ name, passed: false, message: error.message, stack: error.stack, consoleErrors, screenshot, domPath, milliseconds: Math.round(performance.now() - started) });
    console.error(`FAIL ${name}: ${error.message}`);
  } finally {
    await context.close();
  }
}

await test('default fixture, localized names and stable IDs after app rename/optional field', async page => {
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Ada');
  await valueIs(page, 'cell-rec_customer_1-fld_customer_owner', 'actor_alice');
  assert.equal(await byId(page, 'save').getAttribute('disabled'), null);
  assert.equal(await byId(page, 'save').textContent(), 'Giữ nháp trên thiết bị');
  assert.equal(await byId(page, 'cell-rec_customer_1-fld_customer_name').getAttribute('aria-label'), 'Name · Bản ghi');
  await english(page);
  assert.equal(await byId(page, 'save').textContent(), 'Save local draft');
  assert.equal(await byId(page, 'cell-rec_customer_1-fld_customer_name').getAttribute('aria-label'), 'Name · Record');
  await byId(page, 'rename-app').fill('');
  await byId(page, 'save').click();
  await attributeIs(page, 'rename-app', 'aria-invalid', 'true');
  await focused(page, 'rename-app');
  await valueIs(page, 'rename-app', '');
  assert.equal(await byId(page, 'app-name').textContent(), 'Commerce');
  const nameDiagnostic = await byId(page, 'rename-app').getAttribute('aria-describedby');
  assert.ok((await page.locator(`[id="${nameDiagnostic}"]`).textContent()).length > 0);
  assert.equal(await rawStorage(page), null);
  await byId(page, 'rename-app').fill('Ứng dụng thương mại dài để kiểm tra nhãn');
  await byId(page, 'rename-app').press('Tab');
  await textIncludes(page, 'app-name', 'Ứng dụng thương mại dài để kiểm tra nhãn');
  await byId(page, 'field-name').fill('Ghi chú bổ sung cho khách hàng');
  await byId(page, 'field-type').selectOption('text');
  await byId(page, 'add-field').click();
  await valueIs(page, 'cell-rec_customer_1-fld_local_1', '');
  await byId(page, 'cell-rec_customer_1-fld_local_1').fill('Đã gọi điện');
  await byId(page, 'cell-rec_customer_1-fld_local_1').press('Enter');
  await select(page, 'tbl_products');
  await byId(page, 'field-name').fill('Ghi chú riêng cho sản phẩm');
  await byId(page, 'add-field').click();
  await valueIs(page, 'cell-rec_product_1-fld_local_2', '');
  assert.equal(await byId(page, 'cell-rec_product_1-fld_local_1').count(), 0);
  await select(page, 'tbl_customers');
  await save(page);
  const receipt = await envelope(page);
  const definition = JSON.parse(receipt.definition_json);
  assert.equal(definition.app_id, 'app_commerce');
  assert.equal(definition.name, 'Ứng dụng thương mại dài để kiểm tra nhãn');
  assert.deepEqual(definition.tables.map(table => table.id).sort(), ['tbl_customers', 'tbl_order_lines', 'tbl_orders', 'tbl_products']);
  assert.deepEqual(definition.tables[0].fields.map(field => field.id), ['fld_customer_name', 'fld_customer_owner', 'fld_local_1']);
  const fieldIds = definition.tables.flatMap(table => table.fields.map(field => field.id));
  assert.equal(new Set(fieldIds).size, fieldIds.length, 'new optional field identities are unique across tables');
  assert.ok(definition.tables.find(table => table.id === 'tbl_products').fields.some(field => field.id === 'fld_local_2'));
  assert.equal(stringField(receipt, 'rec_customer_1', 'fld_local_1'), 'Đã gọi điện');
  await page.reload({ waitUntil: 'networkidle' });
  await valueIs(page, 'cell-rec_customer_1-fld_local_1', 'Đã gọi điện');
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Ada');
});

await test('invalid typed import is atomic and identifies row/column while preserving source', async page => {
  await english(page);
  await select(page, 'tbl_products');
  const source = 'fld_product_name\tfld_product_price\nValid candidate\t1234\nInvalid candidate\t12.5';
  await byId(page, 'import-source').fill(source);
  await byId(page, 'import-apply').click();
  await byId(page, 'import-errors').waitFor({ state: 'visible' });
  assert.match(await byId(page, 'import-errors').textContent(), /Row 3, column 2.*Price/);
  await rows(page, 1);
  await valueIs(page, 'cell-rec_product_1-fld_product_price', '1250');
  await valueIs(page, 'import-source', source);
  assert.equal(await rawStorage(page), null);
  await byId(page, 'import-source').fill('fld_unknown\nanything');
  await byId(page, 'import-apply').click();
  assert.match(await byId(page, 'import-errors').textContent(), /Column 1.*unknown or duplicate/);
  await rows(page, 1);
});

await test('missing required import field reports imported row and missing column', async page => {
  await english(page);
  const source = 'fld_customer_name\nNew customer';
  await byId(page, 'import-source').fill(source);
  await byId(page, 'import-apply').click();
  await byId(page, 'import-errors').waitFor({ state: 'visible' });
  const diagnostic = await byId(page, 'import-errors').textContent();
  assert.match(diagnostic, /Row 2/);
  assert.match(diagnostic, /column missing.*Owner/i);
  await rows(page, 1);
  await valueIs(page, 'import-source', source);
});

await test('valid required-column import allows omitted optional field and saves all rows', async page => {
  await english(page);
  await select(page, 'tbl_orders');
  await valueIs(page, 'cell-rec_order_1-fld_order_notes', '');
  await byId(page, 'import-source').fill('fld_order_customer\tfld_order_owner\tfld_order_state\nrec_customer_1\tactor_alice\tdraft\nrec_customer_1\tactor_bob\tdraft');
  await byId(page, 'import-apply').click();
  await rows(page, 3);
  assert.equal(await byId(page, 'import-errors').count(), 0);
  await valueIs(page, 'cell-rec_import_1-fld_order_notes', '');
  await valueIs(page, 'cell-rec_import_2-fld_order_notes', '');
  await save(page);
  assert.match((await envelope(page)).records_json, /"id":"rec_import_1"/);
  assert.match((await envelope(page)).records_json, /"id":"rec_import_2"/);
  await page.reload({ waitUntil: 'networkidle' });
  await select(page, 'tbl_orders');
  await rows(page, 3);
});

await test('canonical text constraint authoring validates existing rows and governs grid form import save and cancel', async page => {
  await english(page);
  await byId(page, 'constraint-field').selectOption('fld_customer_name');
  await byId(page, 'constraint-kind').selectOption('text_length');
  await byId(page, 'constraint-min').fill('1');
  await byId(page, 'constraint-max').fill('2');
  await byId(page, 'constraint-add').click();
  await textIncludes(page, 'constraint-status', 'Rules were not changed');
  assert.equal(await byId(page, 'constraint-source').locator('li').count(), 0, 'a rule rejecting existing Ada is not applied');
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Ada');
  assert.equal(await rawStorage(page), null);
  await byId(page, 'constraint-max').fill('4');
  await byId(page, 'constraint-add').click();
  await textIncludes(page, 'constraint-source', 'Character length: 1–4');
  await byId(page, 'cell-rec_customer_1-fld_customer_name').fill('Too long');
  await byId(page, 'cell-rec_customer_1-fld_customer_name').press('Enter');
  await attributeIs(page, 'cell-rec_customer_1-fld_customer_name', 'aria-invalid', 'true');
  await byId(page, 'view-form').click();
  await valueIs(page, 'form-rec_customer_1-fld_customer_name', 'Too long');
  await attributeIs(page, 'form-rec_customer_1-fld_customer_name', 'aria-invalid', 'true');
  await byId(page, 'form-rec_customer_1-fld_customer_name').fill('Eve');
  await byId(page, 'form-rec_customer_1-fld_customer_name').press('Enter');
  await byId(page, 'view-table').click();
  await byId(page, 'import-source').fill('fld_customer_name\tfld_customer_owner\nToo long\tactor_alice');
  await byId(page, 'import-apply').click();
  await rows(page, 1);
  await textIncludes(page, 'import-errors', 'Row 2, column 1 (Name)');
  await textIncludes(page, 'import-errors', 'data rule');
  await valueIs(page, 'import-source', 'fld_customer_name\tfld_customer_owner\nToo long\tactor_alice');
  await save(page);
  const stored = await envelope(page);
  assert.match(stored.definition_json, /"version":2/);
  assert.match(stored.definition_json, /"constraint_id":"constraint_local_1"/);
  assert.match(stored.definition_json, /"kind":"text_length","min":1,"max":4/);
  await page.reload({ waitUntil: 'networkidle' });
  await english(page);
  await textIncludes(page, 'constraint-source', 'Character length: 1–4');
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Eve');
  await byId(page, 'constraint-remove-constraint_local_1').click();
  assert.equal(await byId(page, 'constraint-source').locator('li').count(), 0);
  await byId(page, 'cancel').click();
  await textIncludes(page, 'constraint-source', 'Character length: 1–4');
  await byId(page, 'session-toggle').click();
  assert.equal(await byId(page, 'constraint-add').isDisabled(), true);
  assert.equal(await byId(page, 'constraint-remove-constraint_local_1').isDisabled(), true);
  await byId(page, 'session-toggle').click();
  await byId(page, 'locale').click();
  await textIncludes(page, 'constraint-source', 'Độ dài ký tự: 1–4');
  await captureScenario(page, 'constraints-vi-light', 'Canonical text length rule survives receipt reload and Cancel; grid, form and import use the same Rust constraint evaluator.');
});

await test('numeric constraint authoring retains exact i64 bounds and rejects invalid range without mutation', async page => {
  await english(page);
  await select(page, 'tbl_order_lines');
  await byId(page, 'constraint-field').selectOption('fld_line_quantity');
  await byId(page, 'constraint-min').fill('3');
  await byId(page, 'constraint-max').fill('2');
  await byId(page, 'constraint-add').click();
  await textIncludes(page, 'constraint-status', 'Rules were not changed');
  assert.equal(await byId(page, 'constraint-source').locator('li').count(), 0);
  await byId(page, 'constraint-min').fill('2');
  await byId(page, 'constraint-max').fill('9223372036854775807');
  await byId(page, 'constraint-add').click();
  await textIncludes(page, 'constraint-status', 'Rule added');
  await byId(page, 'cell-rec_line_1-fld_line_quantity').fill('1');
  await byId(page, 'cell-rec_line_1-fld_line_quantity').press('Enter');
  await attributeIs(page, 'cell-rec_line_1-fld_line_quantity', 'aria-invalid', 'true');
  await byId(page, 'cell-rec_line_1-fld_line_quantity').press('Escape');
  await save(page);
  assert.match((await envelope(page)).definition_json, /"kind":"numeric_range","min":2,"max":9223372036854775807/);
  await page.reload({ waitUntil: 'networkidle' });
  await english(page);
  await select(page, 'tbl_order_lines');
  await textIncludes(page, 'constraint-source', '9223372036854775807');
  await valueIs(page, 'cell-rec_line_1-fld_line_quantity', '2');
});

await test('typed diagnostics retain invalid input/focus; Escape cancels; Enter/Tab retain useful focus', async page => {
  await english(page);
  await select(page, 'tbl_order_lines');
  const quantity = 'cell-rec_line_1-fld_line_quantity';
  const price = 'cell-rec_line_1-fld_line_price';
  await byId(page, quantity).fill('1.5');
  await byId(page, quantity).press('Enter');
  await attributeIs(page, quantity, 'aria-invalid', 'true');
  await valueIs(page, quantity, '1.5');
  await focused(page, quantity);
  const description = await byId(page, quantity).getAttribute('aria-describedby');
  assert.match(await page.locator(`[id="${description}"]`).textContent(), /Expected input type: integer/);
  await byId(page, quantity).press('Tab');
  await focused(page, quantity);
  await select(page, 'tbl_products');
  await byId(page, 'save').click();
  await attributeIs(page, 'table-tbl_order_lines', 'aria-pressed', 'true');
  await focused(page, quantity);
  await valueIs(page, quantity, '1.5');
  await attributeIs(page, quantity, 'aria-invalid', 'true');
  assert.equal(await rawStorage(page), null);
  await byId(page, quantity).press('Escape');
  await valueIs(page, quantity, '2');
  await attributeIs(page, quantity, 'aria-invalid', 'false');
  await byId(page, quantity).fill('3');
  await byId(page, quantity).press('Enter');
  await valueIs(page, quantity, '3');
  await focused(page, quantity);
  await byId(page, quantity).press('Tab');
  await focused(page, price);
  await byId(page, 'view-form').click();
  await valueIs(page, 'form-rec_line_1-fld_line_quantity', '3');
  await byId(page, 'form-rec_line_1-fld_line_quantity').fill('4');
  await byId(page, 'form-rec_line_1-fld_line_quantity').press('Enter');
  await byId(page, 'view-table').click();
  await valueIs(page, quantity, '4');
});

await test('signed i64 money is persisted and reloaded without JavaScript numeric conversion', async page => {
  await select(page, 'tbl_products');
  const price = 'cell-rec_product_1-fld_product_price';
  const exact = '9223372036854775807';
  await byId(page, price).fill(exact);
  await byId(page, price).press('Enter');
  await valueIs(page, price, exact);
  await save(page);
  const receipt = await envelope(page);
  assert.ok(receipt.records_json.includes(`"fld_product_price":{"type":"money","value":${exact}}`), 'persisted money retains every i64 digit');
  await page.reload({ waitUntil: 'networkidle' });
  await select(page, 'tbl_products');
  await valueIs(page, price, exact);
  await byId(page, price).fill('9223372036854775808');
  await byId(page, price).press('Enter');
  await attributeIs(page, price, 'aria-invalid', 'true');
  await valueIs(page, price, '9223372036854775808');
  await byId(page, price).press('Escape');
  await valueIs(page, price, exact);
});

await test('DOM composition boundaries prevent premature Enter/Save commit and preserve Vietnamese text', async page => {
  const name = 'cell-rec_customer_1-fld_customer_name';
  await byId(page, name).focus();
  await byId(page, name).dispatchEvent('compositionstart', { data: '' });
  await byId(page, name).fill('Đặng Thị Hồng');
  await byId(page, name).press('Enter');
  assert.equal(await byId(page, 'save').isDisabled(), true);
  assert.equal(await rawStorage(page), null);
  await valueIs(page, name, 'Đặng Thị Hồng');
  await byId(page, name).dispatchEvent('compositionend', { data: 'Đặng Thị Hồng' });
  await byId(page, name).press('Escape');
  await valueIs(page, name, 'Ada'); // Enter during composition did not change committed records.
  await byId(page, name).dispatchEvent('compositionstart', { data: '' });
  await byId(page, name).fill('Đặng Thị Hồng');
  await byId(page, name).dispatchEvent('compositionend', { data: 'Đặng Thị Hồng' });
  await byId(page, name).press('Enter');
  await save(page);
  assert.equal(stringField(await envelope(page), 'rec_customer_1', 'fld_customer_name'), 'Đặng Thị Hồng');
});

await test('repeated local save has one content revision, survives reload, and exposes local receipt', async page => {
  await english(page);
  await byId(page, 'cell-rec_customer_1-fld_customer_name').fill('Grace');
  await save(page);
  const firstRaw = await rawStorage(page);
  const first = await envelope(page);
  assert.equal(first.revision, '1');
  assert.equal(stringField(first, 'rec_customer_1', 'fld_customer_name'), 'Grace');
  await save(page);
  assert.equal(await rawStorage(page), firstRaw);
  assert.equal((await envelope(page)).revision, '1');
  await textIncludes(page, 'draft-status', 'not synchronized to a server');
  await page.reload({ waitUntil: 'networkidle' });
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Grace');
});

await test('Cancel restores saved values and definition rather than leaving unsaved structure', async page => {
  await save(page);
  const baseline = await rawStorage(page);
  await byId(page, 'cell-rec_customer_1-fld_customer_name').fill('Discard this');
  await byId(page, 'cell-rec_customer_1-fld_customer_name').press('Enter');
  await byId(page, 'rename-app').fill('Discarded application');
  await byId(page, 'rename-app').press('Tab');
  await byId(page, 'field-name').fill('Discarded field');
  await byId(page, 'add-field').click();
  await valueIs(page, 'cell-rec_customer_1-fld_local_1', '');
  await byId(page, 'cancel').click();
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Ada');
  assert.equal(await byId(page, 'app-name').textContent(), 'Commerce');
  assert.equal(await byId(page, 'cell-rec_customer_1-fld_local_1').count(), 0);
  assert.equal(await rawStorage(page), baseline);
});

await test('simulated session expiry blocks save and retains draft until resume', async page => {
  await english(page);
  await byId(page, 'cell-rec_customer_1-fld_customer_name').fill('Session draft');
  await page.locator('details').filter({ has: byId(page, 'session-toggle') }).locator('summary').click();
  await byId(page, 'session-toggle').click();
  await textIncludes(page, 'draft-status', 'Simulated expired session');
  assert.equal(await byId(page, 'save').isDisabled(), true);
  assert.equal(await byId(page, 'import-apply').isDisabled(), true);
  assert.equal(await byId(page, 'add-field').isDisabled(), true);
  await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Session draft');
  assert.equal(await rawStorage(page), null);
  await byId(page, 'session-toggle').click();
  assert.equal(await byId(page, 'save').isDisabled(), false);
  await save(page);
  assert.equal(stringField(await envelope(page), 'rec_customer_1', 'fld_customer_name'), 'Session draft');
});

await test('second-tab stale save preserves its draft and cannot overwrite the newer local receipt', async (page, context) => {
  await english(page);
  const other = await context.newPage();
  await open(other);
  await english(other);
  await byId(page, 'cell-rec_customer_1-fld_customer_name').fill('First tab');
  await save(page);
  const receipt = await rawStorage(page);
  await byId(other, 'cell-rec_customer_1-fld_customer_name').fill('Second tab draft');
  await byId(other, 'save').click();
  await textIncludes(other, 'draft-status', 'draft changed in another tab');
  await valueIs(other, 'cell-rec_customer_1-fld_customer_name', 'Second tab draft');
  assert.equal(await rawStorage(other), receipt);
  await other.reload({ waitUntil: 'networkidle' });
  await valueIs(other, 'cell-rec_customer_1-fld_customer_name', 'First tab');
});

await test('simultaneous two-tab saves create exactly one receipt and one conflict without losing either intent', async (page, context) => {
  await english(page);
  const other = await context.newPage();
  await open(other);
  await english(other);
  const name = 'cell-rec_customer_1-fld_customer_name';
  await byId(page, name).fill('Tab A race');
  await byId(other, name).fill('Tab B race');
  await Promise.all([byId(page, 'save').click(), byId(other, 'save').click()]);
  const finished = candidate => candidate.waitForFunction(() => {
    const state = document.querySelector('[data-testid="draft-status"]')?.getAttribute('data-state');
    return (state === 'saved' || state === 'error') && !document.querySelector('[data-testid="save"]')?.disabled;
  });
  await Promise.all([finished(page), finished(other)]);
  const states = await Promise.all([byId(page, 'draft-status').getAttribute('data-state'), byId(other, 'draft-status').getAttribute('data-state')]);
  assert.deepEqual(states.slice().sort(), ['error', 'saved']);
  const loser = states[0] === 'error' ? page : other;
  await textIncludes(loser, 'draft-status', 'draft changed in another tab');
  await valueIs(page, name, 'Tab A race');
  await valueIs(other, name, 'Tab B race');
  const receipt = await envelope(page);
  assert.equal(receipt.revision, '1');
  assert.equal(stringField(receipt, 'rec_customer_1', 'fld_customer_name'), states[0] === 'saved' ? 'Tab A race' : 'Tab B race');
  assert.equal(await rawStorage(page), await rawStorage(other));
});

await test('corrupt, unsupported and empty-table snapshots remain byte-exact and cannot be overwritten', async page => {
  const emptyDefinition = JSON.stringify({ version: 1, app_id: 'app_empty', name: 'Empty app', tables: [], capture_rules: [] });
  for (const raw of [
    '{',
    JSON.stringify({ format: 99, revision: '0', definition_json: '{}', records_json: '[]' }),
    JSON.stringify({ format: 1, revision: '0', definition_json: emptyDefinition, records_json: '[]' }),
  ]) {
    await page.evaluate(({ key, raw }) => localStorage.setItem(key, raw), { key: storageKey, raw });
    await page.reload({ waitUntil: 'networkidle' });
    await valueIs(page, 'cell-rec_customer_1-fld_customer_name', 'Ada');
    await english(page);
    await textIncludes(page, 'draft-status', 'Local storage could not be read or written');
    assert.equal(await byId(page, 'save').isDisabled(), true);
    assert.equal(await rawStorage(page), raw, 'recovery must preserve the unreadable original bytes');
  }
});

await test('reserved text literals null, empty and text prefix retain their Text type across save/reload', async page => {
  const name = 'cell-rec_customer_1-fld_customer_name';
  for (const [input, literal] of [['text:null', 'null'], ['text:', ''], ['text:text:prefix', 'text:prefix']]) {
    await byId(page, name).fill(input);
    await byId(page, name).press('Enter');
    await save(page);
    assert.equal(stringField(await envelope(page), 'rec_customer_1', 'fld_customer_name'), literal);
    await page.reload({ waitUntil: 'networkidle' });
    await valueIs(page, name, input);
    await attributeIs(page, name, 'aria-invalid', 'false');
  }
});

await test('native hash history Back/Forward retains edited records and invalid buffers until Cancel', async page => {
  const name = 'cell-rec_customer_1-fld_customer_name';
  await save(page);
  const savedBytes = await rawStorage(page);
  await byId(page, name).fill('History draft');
  await byId(page, name).press('Enter');
  await select(page, 'tbl_products');
  assert.equal(new URL(page.url()).hash, '#tbl_products');
  await page.goBack();
  await attributeIs(page, 'table-tbl_customers', 'aria-pressed', 'true');
  await valueIs(page, name, 'History draft');
  await page.goForward();
  await attributeIs(page, 'table-tbl_products', 'aria-pressed', 'true');
  await valueIs(page, 'cell-rec_product_1-fld_product_price', '1250');
  await page.goBack();
  await attributeIs(page, 'table-tbl_customers', 'aria-pressed', 'true');
  await byId(page, name).fill('null');
  await byId(page, name).press('Enter');
  await attributeIs(page, name, 'aria-invalid', 'true');
  await select(page, 'tbl_products');
  await page.goBack();
  await attributeIs(page, 'table-tbl_customers', 'aria-pressed', 'true');
  await valueIs(page, name, 'null');
  await attributeIs(page, name, 'aria-invalid', 'true');
  await page.goForward();
  await attributeIs(page, 'table-tbl_products', 'aria-pressed', 'true');
  await byId(page, 'cancel').click();
  await page.goBack();
  await attributeIs(page, 'table-tbl_customers', 'aria-pressed', 'true');
  await valueIs(page, name, 'Ada');
  await attributeIs(page, name, 'aria-invalid', 'false');
  assert.equal(await rawStorage(page), savedBytes);
});

await test('actual grid capture retains old price, derives a new imported line, and rejects supplied capture atomically', async page => {
  await english(page);
  await select(page, 'tbl_products');
  await byId(page, 'cell-rec_product_1-fld_product_price').fill('1900');
  await byId(page, 'cell-rec_product_1-fld_product_price').press('Enter');
  await select(page, 'tbl_order_lines');
  await valueIs(page, 'cell-rec_line_1-fld_line_price', '1250');
  assert.equal(await byId(page, 'cell-rec_line_1-fld_line_price').evaluate(input => input.readOnly), true);
  assert.equal(await byId(page, 'cell-rec_line_1-fld_line_product').evaluate(input => input.readOnly), true);
  assert.equal(await byId(page, 'cell-rec_line_1-fld_line_quantity').evaluate(input => input.readOnly), false);
  await byId(page, 'cell-rec_line_1-fld_line_quantity').fill('3');
  await byId(page, 'cell-rec_line_1-fld_line_quantity').press('Enter');
  await valueIs(page, 'cell-rec_line_1-fld_line_quantity', '3');
  await captureScenario(page, 'capture-grid-en-light', 'Actual grid: updated product price 1900, historical line price 1250, editable quantity 3; managed fields have their original readonly attributes.');
  await byId(page, 'import-source').fill('fld_line_order\tfld_line_product\tfld_line_quantity\nrec_order_1\trec_product_1\t1');
  await byId(page, 'import-apply').click();
  await rows(page, 2);
  await valueIs(page, 'cell-rec_import_1-fld_line_price', '1900');
  await valueIs(page, 'cell-rec_line_1-fld_line_price', '1250');
  await save(page);
  const accepted = await rawStorage(page);
  const source = 'fld_line_order\tfld_line_product\tfld_line_quantity\tfld_line_price\nrec_order_1\trec_product_1\t1\t7';
  await byId(page, 'import-source').fill(source);
  await byId(page, 'import-apply').click();
  await byId(page, 'import-errors').waitFor({ state: 'visible' });
  assert.match(await byId(page, 'import-errors').textContent(), /Row 2, column 4 \(Captured price\).*derived/);
  await rows(page, 2);
  await valueIs(page, 'import-source', source);
  assert.equal(await rawStorage(page), accepted);
  await page.reload({ waitUntil: 'networkidle' });
  await select(page, 'tbl_order_lines');
  await valueIs(page, 'cell-rec_line_1-fld_line_price', '1250');
  await valueIs(page, 'cell-rec_import_1-fld_line_price', '1900');
  await valueIs(page, 'cell-rec_line_1-fld_line_quantity', '3');
  const receipt = await envelope(page);
  assert.ok(receipt.records_json.includes('"fld_line_price":{"type":"money","value":1250}'));
  assert.ok(receipt.records_json.includes('"fld_line_price":{"type":"money","value":1900}'));
  await select(page, 'tbl_products');
  await valueIs(page, 'cell-rec_product_1-fld_product_price', '1900');
});

await test('capture guards reject historical price and product edits even if readonly is bypassed', async page => {
  await english(page);
  await select(page, 'tbl_products');
  await byId(page, 'import-source').fill('fld_product_name\tfld_product_price\nAlternate product\t2200');
  await byId(page, 'import-apply').click();
  await rows(page, 2);
  await save(page);
  const baseline = await rawStorage(page);
  await select(page, 'tbl_order_lines');
  const price = 'cell-rec_line_1-fld_line_price';
  const product = 'cell-rec_line_1-fld_line_product';
  for (const [id, attempted, original, diagnostic] of [
    [price, '7', '1250', /Captured historical prices cannot be edited/],
    [product, 'rec_import_1', 'rec_product_1', /Create a new line to choose another product/],
  ]) {
    assert.equal(await byId(page, id).evaluate(input => input.readOnly), true);
    // Deliberately bypass only the presentation restriction to exercise the Rust guard.
    // No screenshots are captured while renderer-owned attributes are changed.
    await byId(page, id).evaluate(input => input.removeAttribute('readonly'));
    await byId(page, id).fill(attempted);
    await byId(page, id).press('Enter');
    await attributeIs(page, id, 'aria-invalid', 'true');
    await valueIs(page, id, attempted);
    const errorId = (await byId(page, id).getAttribute('aria-describedby')).split(' ')[0];
    assert.match(await page.locator(`[id="${errorId}"]`).textContent(), diagnostic);
    await byId(page, 'save').click();
    await attributeIs(page, 'draft-status', 'data-state', 'error');
    assert.equal(await rawStorage(page), baseline);
    await byId(page, id).press('Escape');
    await valueIs(page, id, original);
    await attributeIs(page, id, 'aria-invalid', 'false');
    await byId(page, id).evaluate(input => input.setAttribute('readonly', ''));
  }
  await save(page);
  assert.equal(await rawStorage(page), baseline);
  await page.reload({ waitUntil: 'networkidle' });
  await select(page, 'tbl_order_lines');
  await valueIs(page, price, '1250');
  await valueIs(page, product, 'rec_product_1');
  assert.equal(await byId(page, price).evaluate(input => input.readOnly), true);
  assert.equal(await byId(page, product).evaluate(input => input.readOnly), true);
});

await test('relation laboratory projects live/captured totals, restricts delete, and localizes retained error state', async page => {
  await english(page);
  await relationLine(page, 'rec_line_1', ['1250', '1250', '2500']);
  await textIncludes(page, 'relation-total', 'Historical total: 2500');
  await byId(page, 'relation-price').fill('2000');
  await byId(page, 'relation-change-price').click();
  await textIncludes(page, 'relation-status', 'older lines keep their captured price');
  await relationLine(page, 'rec_line_1', ['2000', '1250', '2500']);
  await byId(page, 'relation-add-line').click();
  await relationLine(page, 'rec_line_2', ['2000', '2000', '2000']);
  await textIncludes(page, 'relation-total', 'Historical total: 4500');
  await captureScenario(page, 'relation-normal-en-light', 'Relation laboratory: existing captured price 1250, new captured price 2000, current lookup 2000 and historical total 4500.');
  const previous = await byId(page, 'relation-lines').locator('tbody').textContent();
  await byId(page, 'relation-delete-product').click();
  await textIncludes(page, 'relation-status', 'Cannot delete: a line still references');
  assert.match(await byId(page, 'relation-status').textContent(), /rec_line_1 \/ fld_line_product/);
  assert.equal(await byId(page, 'relation-lines').locator('tbody').textContent(), previous);
  await textIncludes(page, 'relation-total', 'Historical total: 4500');
  await byId(page, 'relation-price').fill('abc');
  await byId(page, 'relation-change-price').click();
  await attributeIs(page, 'relation-price', 'aria-invalid', 'true');
  await textIncludes(page, 'relation-status', 'integer minor units');
  await valueIs(page, 'relation-price', 'abc');
  assert.equal(await byId(page, 'relation-lines').locator('tbody').textContent(), previous);
  await byId(page, 'locale').click();
  await textIncludes(page, 'relation-status', 'Nhập giá bằng số nguyên');
  await textIncludes(page, 'relation-total', 'Tổng lịch sử: 4500');
  assert.match(await byId(page, 'relation-lines').locator('thead').textContent(), /Giá hiện tại.*Giá đã chốt/);
  await captureScenario(page, 'relation-invalid-vi-light', 'Relation laboratory: invalid price abc is retained, prior derived results remain, localized Vietnamese typed feedback is visible.');
  await byId(page, 'locale').click();
  await textIncludes(page, 'relation-status', 'integer minor units');
  assert.equal(await rawStorage(page), null, 'the relation laboratory does not claim to persist the independent Builder draft');
});

await test('policy source is canonical, survives save/reload, cancels correctly, and blocks source edits during pause/pending save', async page => {
  await english(page);
  await select(page, 'tbl_orders');
  await textIncludes(page, 'policy-source', 'No rule for this table');
  await save(page);
  await byId(page, 'policy-add-source').click();
  await textIncludes(page, 'policy-source', 'rule_order_owner · v1');
  await textIncludes(page, 'draft-status', 'Changes are not yet saved');
  await save(page);
  const addedBytes = await rawStorage(page);
  let receipt = await envelope(page);
  let definition = JSON.parse(receipt.definition_json);
  assert.equal(receipt.revision, '1');
  assert.equal(definition.policies.length, 1);
  assert.equal(definition.policies[0].rule_id, 'rule_order_owner');
  assert.equal(definition.policies[0].owner_field, 'fld_order_owner');
  assert.deepEqual(definition.policies[0].submit_roles, ['role_operator']);
  assert.equal(definition.submit_rules[0].policy_rule_id, 'rule_order_owner');
  assert.equal(definition.submit_rules[0].quantity_field, 'fld_line_quantity');
  await byId(page, 'policy-allow-operator').uncheck();
  await textIncludes(page, 'policy-source', 'v2');
  await textIncludes(page, 'draft-status', 'Changes are not yet saved');
  await byId(page, 'cancel').click();
  await textIncludes(page, 'policy-source', 'v1');
  assert.equal(await byId(page, 'policy-allow-operator').isChecked(), true);
  assert.equal(await rawStorage(page), addedBytes);
  await byId(page, 'policy-allow-operator').uncheck();
  await save(page);
  receipt = await envelope(page);
  assert.equal(receipt.revision, '2');
  assert.deepEqual(JSON.parse(receipt.definition_json).policies[0].submit_roles, []);
  await page.reload({ waitUntil: 'networkidle' });
  await english(page);
  await select(page, 'tbl_orders');
  await textIncludes(page, 'policy-source', 'v2');
  assert.equal(await byId(page, 'policy-allow-operator').isChecked(), false);
  const persisted = await rawStorage(page);
  await page.locator('details').filter({ has: byId(page, 'session-toggle') }).locator('summary').click();
  await byId(page, 'session-toggle').click();
  for (const id of ['policy-add-source', 'policy-allow-auditor', 'policy-allow-operator']) assert.equal(await byId(page, id).isDisabled(), true);
  assert.equal(await rawStorage(page), persisted);
  await byId(page, 'session-toggle').click();
  await byId(page, 'policy-allow-auditor').uncheck();
  await textIncludes(page, 'policy-source', 'v3');
  await page.evaluate(() => {
    window.__qaLockEntered = false;
    window.__qaLockRelease = null;
    window.__qaLockPromise = navigator.locks.request('evobase-builder-local-v1', async () => {
      window.__qaLockEntered = true;
      await new Promise(resolve => { window.__qaLockRelease = resolve; });
    });
  });
  await page.waitForFunction(() => window.__qaLockEntered);
  try {
    await byId(page, 'save').click();
    await textIncludes(page, 'draft-status', 'Saving local draft');
    for (const id of ['policy-add-source', 'policy-allow-auditor', 'policy-allow-operator']) assert.equal(await byId(page, id).isDisabled(), true);
    assert.equal(await rawStorage(page), persisted, 'a queued save has no premature storage receipt');
  } finally {
    await page.evaluate(() => window.__qaLockRelease());
  }
  await page.waitForFunction(() => document.querySelector('[data-testid="draft-status"]')?.getAttribute('data-state') === 'saved' && !document.querySelector('[data-testid="save"]')?.disabled);
  definition = JSON.parse((await envelope(page)).definition_json);
  assert.deepEqual(definition.policies[0].read_roles, []);
  assert.deepEqual(definition.policies[0].submit_roles, []);
});

await test('policy Read and Submit grants are independent and a non-owner needs the allowed current role', async page => {
  await addOrderRule(page);
  await textIncludes(page, 'policy-visible', 'Readable records: 1 [rec_order_1]');
  await byId(page, 'policy-grant-Read').uncheck();
  for (const grant of ['Design', 'Publish', 'Manage', 'Write']) await byId(page, `policy-grant-${grant}`).check();
  await textIncludes(page, 'policy-visible', 'current business grant is missing');
  await byId(page, 'policy-grant-Read').check();
  await textIncludes(page, 'policy-visible', 'Readable records: 1');
  await byId(page, 'policy-grant-Submit').uncheck();
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'current business grant is missing');
  await byId(page, 'policy-grant-Submit').check();
  await byId(page, 'policy-actor').selectOption('actor_bob');
  await byId(page, 'policy-role').selectOption('role_operator');
  await byId(page, 'policy-allow-operator').uncheck();
  await textIncludes(page, 'policy-visible', 'Readable records: 0 []');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'owner/role rule does not permit');
  await byId(page, 'policy-role').selectOption('role_auditor');
  await textIncludes(page, 'policy-visible', 'Readable records: 1 [rec_order_1]');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'owner/role rule does not permit');
  await byId(page, 'policy-role').selectOption('role_operator');
  await byId(page, 'policy-allow-operator').check();
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'Simulated 2 checked writes; state submitted.');
  await textIncludes(page, 'policy-visible', 'Readable records: 0 []');
  await valueIs(page, 'cell-rec_order_1-fld_order_state', 'draft');
  await captureScenario(page, 'policy-normal-en-light', 'Local simulation: Bob has an allowed Operator Submit role; the checked command applies, while Read remains separately denied and the independent Builder draft stays draft.');
  assert.equal(await rawStorage(page), null, 'simulation does not claim a durable command receipt');
});

await test('policy validates retained input and normalized retry intent, then rechecks revoked grants/membership before replay', async page => {
  await addOrderRule(page);
  const tooLong = 'x'.repeat(1001);
  await byId(page, 'policy-notes').fill(tooLong);
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'Invalid input. Check the notes');
  await valueIs(page, 'policy-notes', tooLong);
  await byId(page, 'policy-notes').fill('  urgent  ');
  await byId(page, 'policy-quantity').fill('0');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'Quantity must be an integer from 1 to 10000');
  await valueIs(page, 'policy-quantity', '0');
  await byId(page, 'locale').click();
  await textIncludes(page, 'policy-decision', 'Số lượng phải là số nguyên từ 1 đến 10000');
  await captureScenario(page, 'policy-invalid-vi-light', 'Local simulation: invalid quantity 0 remains in the input with Vietnamese feedback; no command writes or durable receipts are asserted.');
  await byId(page, 'locale').click();
  await byId(page, 'policy-quantity').fill('1.5');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'Quantity must be an integer from 1 to 10000');
  await valueIs(page, 'policy-quantity', '1.5');
  await byId(page, 'policy-quantity').fill('3');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'Simulated 2 checked writes; state submitted.');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'replayed the simulated receipt');
  await byId(page, 'policy-quantity').fill('4');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'key belongs to a different intent');
  await valueIs(page, 'policy-quantity', '4');
  await byId(page, 'policy-quantity').fill('3');
  await byId(page, 'policy-notes').fill('urgent');
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'replayed the simulated receipt');
  await byId(page, 'policy-active').uncheck();
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'current membership is revoked');
  await textIncludes(page, 'policy-visible', 'current membership is revoked');
  await byId(page, 'policy-active').check();
  await byId(page, 'policy-grant-Submit').uncheck();
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'current business grant is missing');
  await byId(page, 'policy-grant-Submit').check();
  await byId(page, 'policy-submit').click();
  await textIncludes(page, 'policy-decision', 'replayed the simulated receipt');
  await valueIs(page, 'cell-rec_order_1-fld_order_state', 'draft');
  assert.equal(await rawStorage(page), null);
});

await test('VI/EN × light/dark desktop and compact handoff have no axe violations or page overflow', async page => {
  const matrixFailures = [];
  for (const scenario of [
    { locale: 'vi', theme: 'light', width: 1440, height: 1000 },
    { locale: 'en', theme: 'light', width: 1440, height: 1000 },
    { locale: 'en', theme: 'dark', width: 1440, height: 1000 },
    { locale: 'vi', theme: 'dark', width: 1440, height: 1000 },
    { locale: 'vi', theme: 'light', width: 390, height: 844 },
    { locale: 'en', theme: 'dark', width: 390, height: 844 },
  ]) {
    await page.setViewportSize({ width: scenario.width, height: scenario.height });
    if (await page.locator('html').getAttribute('lang') !== scenario.locale) await byId(page, 'locale').click();
    if ((await page.locator('html').getAttribute('data-theme') ?? 'light') !== scenario.theme) await byId(page, 'theme').click();
    assert.equal(await page.locator('html').getAttribute('lang'), scenario.locale);
    assert.equal(await byId(page, 'theme').getAttribute('aria-pressed'), String(scenario.theme === 'dark'));
    const compact = scenario.width < 600;
    assert.equal(await byId(page, 'compact-handoff').isVisible(), compact);
    assert.equal(await byId(page, 'save').isVisible(), !compact);
    assert.equal(await byId(page, 'typed-table').isVisible(), !compact);
    if (compact) assert.match(await byId(page, 'compact-handoff').textContent(), scenario.locale === 'vi' ? /màn hình rộng/ : /wider screen/);
    const geometry = await page.evaluate(() => ({ scrollWidth: document.documentElement.scrollWidth, clientWidth: document.documentElement.clientWidth }));
    const name = `builder-${scenario.locale}-${scenario.theme}-${compact ? 'compact' : 'desktop'}`;
    if (geometry.scrollWidth > geometry.clientWidth + 1) matrixFailures.push({ name, overflow: geometry });
    const screenshot = path.join(evidenceDir, `${name}.png`);
    await page.screenshot({ path: screenshot, fullPage: true });
    const bytes = await readFile(screenshot);
    const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
    await writeFile(path.join(evidenceDir, `${name}-axe.json`), JSON.stringify(axe, null, 2));
    captures.push({ name, ...scenario, origin: 'browser-runtime', path: screenshot, bytes: bytes.length, sha256: digest(bytes), geometry, axeViolations: axe.violations.length, inspected: false });
    if (axe.violations.length) matrixFailures.push({ name, violations: axe.violations.map(({ id, impact, nodes }) => ({ id, impact, targets: nodes.map(node => node.target) })) });
  }
  assert.deepEqual(matrixFailures, [], 'all captured variants reflow without overflow and have no automated axe violations');
});

await browser.close();
const passedScenario = pattern => results.some(result => result.passed && pattern.test(result.name));
const axeScans = captures.filter(capture => Number.isInteger(capture.axeViolations)).length;
const report = {
  source, baseURL, browser: `Chromium ${browser.version()} headless / Playwright`,
  executedAt: new Date().toISOString(),
  artifacts: evidenceDir,
  scope: 'Mounted local-only Leptos Builder with synthetic Commerce fixture and actual localStorage receipts.',
  boundaries: [
    ...(results.some(result => result.passed) ? ['DOM-tested'] : []),
    ...(passedScenario(/typed diagnostics|composition|history|capture guards|actual grid capture|signed i64|reserved text|stable IDs|Cancel/) ? ['keyboard actions'] : []),
    ...(passedScenario(/DOM composition/) ? ['synthetic DOM composition events'] : []),
    ...(passedScenario(/save|reload|import|storage|Cancel|capture|policy|session|receipt|composition|fixture|relation/) ? ['local browser storage read-back'] : []),
    ...(axeScans ? ['automated axe scan'] : []),
    ...(captures.length ? ['browser-runtime captures'] : []),
  ],
  notCovered: ['Real operating-system IME', 'screen reader', 'Firefox/Safari', 'CMP/native runtime', 'server persistence or authentication', 'automated visual comparator; captures require opening before claiming inspected pixels'],
  resources: Object.fromEntries(resourceDigests),
  resourceErrors,
  axeScans,
  scenarioFilter: process.env.SCENARIO_FILTER ?? null,
  skipped,
  results, captures,
  passed: results.filter(result => result.passed).length,
  failed: results.filter(result => !result.passed).length,
};
const reportPath = path.join(evidenceDir, 'builder-browser-report.json');
await writeFile(reportPath, JSON.stringify(report, null, 2));
console.log(`${report.passed}/${results.length} scenarios passed (${skipped} filtered out); report: ${reportPath}`);
if (report.failed || !results.length) process.exitCode = 1;
