// Actual single-tenant HTTP Runtime evidence against two fresh local pilot hosts.
// The access file must be synthetic test configuration; it is restored after denial vectors.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';

function required(name) {
  const value = process.env[name];
  assert.ok(value, `${name} must be supplied`);
  return value;
}
const { chromium } = await import(pathToFileURL(required('PLAYWRIGHT_MODULE_PATH')).href);
const { default: AxeBuilder } = await import(pathToFileURL(required('AXE_MODULE_PATH')).href);
const runtimeUrl = required('RUNTIME_URL');
const supportHost = required('RUNTIME_SUPPORT_HOST');
const libraryHost = required('RUNTIME_LIBRARY_HOST');
const token = required('RUNTIME_TOKEN');
const accessPath = required('RUNTIME_SUPPORT_ACCESS_FILE');
const evidenceDir = required('EVIDENCE_DIR');
await mkdir(evidenceDir, { recursive: true });
const originalAccess = await readFile(accessPath, 'utf8');
const access = JSON.parse(originalAccess);
assert.equal(access.token_sha256, createHash('sha256').update(token).digest('hex'), 'test access file matches the synthetic credential');
const browser = await chromium.launch({ headless: true });
const results = [], captures = [];
const id = (page, testid) => page.locator(`[data-testid="${testid}"]`);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = {
  head: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
  status: execFileSync('git', ['status', '--short'], { cwd: root, encoding: 'utf8' }).trim(),
  files: {},
};
for (const file of ['scripts/check_runtime.mjs', 'scripts/check-runtime.sh', 'Cargo.lock', 'apps/evobase-builder/src/runtime.rs', 'apps/evobase-builder/src/lib.rs', 'apps/evobase-builder/builder.css', 'apps/evobase-builder/dist/evobase_builder.js', 'apps/evobase-builder/dist/evobase_builder_bg.wasm']) {
  source.files[file] = digest(await readFile(path.join(root, file)));
}

async function includes(page, testid, text) {
  await page.waitForFunction(({ testid, text }) => document.querySelector(`[data-testid="${testid}"]`)?.textContent.includes(text), { testid, text });
}
async function english(page) {
  if (await page.locator('html').getAttribute('lang') === 'vi') await id(page, 'runtime-locale').click();
}
async function connection(page) {
  if (!await id(page, 'runtime-server').isVisible()) await page.locator('details').filter({ has: id(page, 'runtime-server') }).locator('summary').click();
}
async function connect(page, host, app) {
  await connection(page);
  await id(page, 'runtime-server').fill(host);
  await id(page, 'runtime-app').fill(app);
  await id(page, 'runtime-credential').fill(token);
  await id(page, 'runtime-connect').click();
  await includes(page, 'runtime-status', 'Server data loaded');
}
async function noStoredSecrets(page) {
  const stored = await page.evaluate(() => ({ local: { ...localStorage }, session: { ...sessionStorage } }));
  assert.deepEqual(stored, { local: {}, session: {} }, 'Runtime stores neither credentials nor server facts in browser persistence');
}
async function capture(page, name, state) {
  const location = path.join(evidenceDir, `${name}.png`);
  await page.screenshot({ path: location, fullPage: true });
  const bytes = await readFile(location);
  captures.push({ name, state, origin: 'browser-runtime', path: location, sha256: digest(bytes), bytes: bytes.length, viewport: page.viewportSize(), locale: await page.locator('html').getAttribute('lang'), theme: await page.locator('html').getAttribute('data-theme'), inspected: false });
}
async function denyCurrentAccess() {
  await writeFile(accessPath, JSON.stringify({ ...access, revoked: true, membership_revision: access.membership_revision + 1 }));
}
async function restoreAccess() { await writeFile(accessPath, originalAccess); }
async function test(name, run) {
  const context = await browser.newContext({ viewport: { width: 1200, height: 900 }, reducedMotion: 'reduce' });
  const page = await context.newPage();
  page.setDefaultTimeout(10000);
  const pageErrors = [];
  page.on('pageerror', error => pageErrors.push(error.message));
  try {
    await page.goto(runtimeUrl, { waitUntil: 'networkidle' });
    await english(page);
    await run(page, context);
    await noStoredSecrets(page);
    assert.deepEqual(pageErrors, [], 'mounted Runtime has no unhandled browser exceptions');
    results.push({ name, passed: true });
    console.log(`PASS ${name}`);
  } catch (error) {
    const screenshot = path.join(evidenceDir, `failure-${results.length + 1}.png`);
    await page.screenshot({ path: screenshot, fullPage: true }).catch(() => {});
    results.push({ name, passed: false, message: error.message, pageErrors, screenshot });
    console.error(`FAIL ${name}: ${error.message}`);
  } finally { await restoreAccess(); await context.close(); }
}

try {
  await test('context switch and Disconnect abort old HTTP reads before their data can enter the new app', async page => {
    let markStarted;
    const started = new Promise(resolve => { markStarted = resolve; });
    let release;
    const delayed = new Promise(resolve => { release = resolve; });
    await page.route(`${supportHost}/v1/apps/app_support/runtime-metadata`, async route => {
      const response = await route.fetch();
      markStarted();
      await delayed;
      await route.fulfill({ response }).catch(() => {});
    });
    await id(page, 'runtime-server').fill(supportHost);
    await id(page, 'runtime-app').fill('app_support');
    await id(page, 'runtime-credential').fill(token);
    await id(page, 'runtime-connect').click();
    await started;
    await connect(page, libraryHost, 'app_library');
    release();
    await id(page, 'runtime-record-rec_loan_1').waitFor();
    assert.equal(await id(page, 'runtime-record-rec_request_1').count(), 0);
    await includes(page, 'runtime-records', 'borrowed');
    await connection(page);
    await id(page, 'runtime-disconnect').click();
    assert.equal(await id(page, 'runtime-credential').inputValue(), '');
    assert.equal(await id(page, 'runtime-records').count(), 0);
    await connect(page, libraryHost, 'app_library');
    await id(page, 'runtime-record-rec_loan_1').waitFor();
  });

  await test('generated support form preserves optional values, checks types and retains denied and stale CAS drafts', async (page, context) => {
    await connect(page, supportHost, 'app_support');
    await id(page, 'runtime-record-rec_request_2').click();
    await id(page, 'runtime-param-fld_request_note').fill('Draft belongs to the other owner');
    await writeFile(accessPath, JSON.stringify({ ...access, roles: access.roles.filter(role => role !== 'role_auditor'), membership_revision: access.membership_revision + 1 }));
    await id(page, 'runtime-refresh').click();
    await includes(page, 'runtime-status', 'Server data loaded');
    assert.equal(await id(page, 'runtime-record-rec_request_2').count(), 0, 'current Read projection hides the other owner record after its read role is removed');
    assert.equal(await id(page, 'runtime-param-fld_request_note').inputValue(), '', 'automatic selection of the remaining owner record cannot inherit the hidden record draft');
    await restoreAccess();
    await id(page, 'runtime-refresh').click();
    await includes(page, 'runtime-status', 'Server data loaded');
    await id(page, 'runtime-record-rec_request_2').waitFor();
    await id(page, 'runtime-record-rec_request_1').click();
    await id(page, 'runtime-param-fld_request_note').fill('Draft belongs to request 1');
    await id(page, 'runtime-record-rec_request_2').click();
    assert.equal(await id(page, 'runtime-param-fld_request_note').inputValue(), '', 'switching records cannot apply the prior record draft');
    await id(page, 'runtime-record-rec_request_1').click();
    await id(page, 'runtime-param-fld_request_note').fill('Reviewed support request');
    await id(page, 'runtime-param-fld_request_priority').fill('1.5');
    await id(page, 'runtime-submit').click();
    assert.equal(await id(page, 'runtime-param-fld_request_priority').getAttribute('aria-invalid'), 'true');
    assert.equal(await id(page, 'runtime-param-fld_request_priority').inputValue(), '1.5');
    let explicitBlank;
    page.on('request', request => { if (request.url() === `${supportHost}/v1/apps/app_support/commands`) explicitBlank = JSON.parse(request.postData()); });
    await id(page, 'runtime-param-fld_request_priority').fill('');
    await id(page, 'runtime-submit').click();
    await includes(page, 'runtime-status', 'does not satisfy');
    assert.equal(explicitBlank.params.fld_request_priority.type, 'blank', 'explicitly cleared optional input is sent, while final record rules still protect required facts');
    await id(page, 'runtime-param-fld_request_priority').fill('2');
    const concurrent = await context.newPage();
    await concurrent.goto(runtimeUrl, { waitUntil: 'networkidle' });
    await english(concurrent);
    await connect(concurrent, supportHost, 'app_support');
    await id(concurrent, 'runtime-record-rec_request_2').click();
    await id(concurrent, 'runtime-param-fld_request_note').fill('Concurrent synthetic edit');
    let untouched;
    concurrent.on('request', request => { if (request.url() === `${supportHost}/v1/apps/app_support/commands`) untouched = JSON.parse(request.postData()); });
    await id(concurrent, 'runtime-submit').click();
    await includes(concurrent, 'runtime-receipt', 'Server confirmed');
    assert.equal(Object.hasOwn(untouched.params, 'fld_request_priority'), false, 'untouched optional input remains absent from the command');
    const listResponse = await concurrent.request.get(`${supportHost}/v1/apps/app_support/tables/tbl_requests?limit=100`, { headers: { Authorization: `Bearer ${token}` } });
    assert.equal(listResponse.status(), 200);
    const advanced = await listResponse.json();
    assert.equal(advanced.records.find(record => record.record_id === 'rec_request_2').values.fld_request_priority.value, 2, 'untouched persisted priority is preserved by the actual host');
    await concurrent.close();
    await id(page, 'runtime-submit').click();
    await includes(page, 'runtime-status', 'The data changed');
    assert.equal(await id(page, 'runtime-param-fld_request_note').inputValue(), 'Reviewed support request');
    assert.equal(await id(page, 'runtime-receipt').count(), 0);
    await id(page, 'runtime-refresh').click();
    await includes(page, 'runtime-status', 'Server data loaded');
    await denyCurrentAccess();
    await id(page, 'runtime-submit').click();
    await includes(page, 'runtime-status', 'Denied:');
    assert.equal(await id(page, 'runtime-records').count(), 0, 'denial clears protected records');
    assert.equal(await id(page, 'runtime-actions').count(), 0, 'denial clears protected actions');
    assert.equal(await id(page, 'runtime-receipt').count(), 0);
    await restoreAccess();
    await id(page, 'runtime-connect').click();
    await includes(page, 'runtime-status', 'Server data loaded');
    assert.equal(await id(page, 'runtime-param-fld_request_note').inputValue(), 'Reviewed support request', 'reauthorization restores the retained same-record draft');
    for (const scenario of [
      { locale: 'en', dark: false, width: 1200, height: 900 },
      { locale: 'en', dark: true, width: 1200, height: 900 },
      { locale: 'vi', dark: true, width: 390, height: 844 },
      { locale: 'vi', dark: false, width: 390, height: 844 },
    ]) {
      await page.setViewportSize({ width: scenario.width, height: scenario.height });
      if (await page.locator('html').getAttribute('lang') !== scenario.locale) await id(page, 'runtime-locale').click();
      if (await id(page, 'runtime-theme').getAttribute('aria-pressed') !== String(scenario.dark)) await id(page, 'runtime-theme').click();
      const geometry = await page.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(geometry.scroll <= geometry.width + 1, 'Runtime reflows without page overflow');
      const axe = await new AxeBuilder({ page }).analyze();
      assert.deepEqual(axe.violations, [], 'generated Runtime has no detected axe violations');
      await capture(page, `runtime-support-${scenario.locale}-${scenario.dark ? 'dark' : 'light'}-${scenario.width}`, 'Actual HTTP-projected support records and generated action with retained input after current permission denial and stale revision conflict.');
    }
  });

  await test('lost command ACK retains one exact request and rechecks current grants before same-key replay', async page => {
    await connect(page, supportHost, 'app_support');
    await id(page, 'runtime-record-rec_request_1').click();
    await id(page, 'runtime-param-fld_request_note').fill('Committed despite lost acknowledgement');
    await id(page, 'runtime-param-fld_request_priority').fill('2');
    const bodies = [];
    let first = true;
    await page.route(`${supportHost}/v1/apps/app_support/commands`, async route => {
      bodies.push(route.request().postData());
      if (first) {
        first = false;
        const response = await route.fetch();
        assert.equal(response.status(), 200, 'real host committed before the ACK was intentionally dropped');
        await route.abort('failed');
      } else { await route.continue(); }
    });
    await id(page, 'runtime-submit').click();
    await includes(page, 'runtime-status', 'outcome is unknown');
    assert.equal(await id(page, 'runtime-submit').isDisabled(), true);
    assert.equal(await id(page, 'runtime-receipt').count(), 0);
    await denyCurrentAccess();
    await id(page, 'runtime-recover').click();
    await includes(page, 'runtime-status', 'Denied:');
    assert.equal(await id(page, 'runtime-records').count(), 0);
    assert.equal(await id(page, 'runtime-actions').count(), 0);
    await id(page, 'runtime-retry').click();
    await includes(page, 'runtime-status', 'Denied:');
    assert.equal(await id(page, 'runtime-retry').count(), 1, 'denied recovery keeps the unresolved immutable request');
    await restoreAccess();
    await id(page, 'runtime-retry').click();
    await includes(page, 'runtime-receipt', 'recovered receipt');
    await includes(page, 'runtime-records', 'resolved');
    assert.equal(bodies.length, 3);
    assert.equal(bodies[1], bodies[0]);
    assert.equal(bodies[2], bodies[0], 'every retry uses the byte-identical request and request key');
    assert.equal(await id(page, 'runtime-retry').count(), 0);
    assert.equal(await id(page, 'runtime-submit').count(), 0, 'terminal record no longer advertises an eligible action');
    await capture(page, 'runtime-support-recovered-en-light', 'Real host commit with a dropped browser ACK; revoked authority denied both recovery and retry; restored authority replayed one exact request and receipt.');
  });

  await test('unrelated library AppSpec uses the same generated list form and action with receipt recovery after Disconnect', async page => {
    await connect(page, libraryHost, 'app_library');
    await id(page, 'runtime-record-rec_loan_1').click();
    assert.equal(await id(page, 'runtime-submit').textContent(), 'Return loan');
    await id(page, 'runtime-param-fld_loan_note').fill('Draft for the first loan');
    await id(page, 'runtime-record-rec_loan_2').click();
    assert.equal(await id(page, 'runtime-param-fld_loan_note').inputValue(), '', 'record switch clears the other record draft');
    const beforeResponse = await page.request.get(`${libraryHost}/v1/apps/app_library/tables/tbl_loans?limit=100`, { headers: { Authorization: `Bearer ${token}` } });
    assert.equal(beforeResponse.status(), 200);
    const before = await beforeResponse.json();
    const storedNote = before.records.find(record => record.record_id === 'rec_loan_2').values.fld_loan_note;
    assert.equal(storedNote.type, 'text', 'the second synthetic loan starts with a distinctive stored text note');
    const optionalRequest = page.waitForRequest(request => request.url() === `${libraryHost}/v1/apps/app_library/commands` && request.method() === 'POST');
    await id(page, 'runtime-submit').click();
    const optionalBody = JSON.parse((await optionalRequest).postData());
    assert.equal(Object.hasOwn(optionalBody.params, 'fld_loan_note'), false, 'untouched optional text is omitted from the actual HTTP request');
    await includes(page, 'runtime-receipt', 'Server confirmed');
    await includes(page, 'runtime-records', 'returned');
    const afterResponse = await page.request.get(`${libraryHost}/v1/apps/app_library/tables/tbl_loans?limit=100`, { headers: { Authorization: `Bearer ${token}` } });
    assert.equal(afterResponse.status(), 200);
    const after = await afterResponse.json();
    assert.deepEqual(after.records.find(record => record.record_id === 'rec_loan_2').values.fld_loan_note, storedNote, 'the real host transition preserves the untouched optional stored note');
    await id(page, 'runtime-record-rec_loan_1').click();
    assert.equal(await id(page, 'runtime-command').inputValue(), 'cmd_return_loan');
    await id(page, 'runtime-param-fld_loan_note').fill('Returned to library desk');
    await id(page, 'runtime-param-fld_loan_days').fill('14');
    await capture(page, 'runtime-library-form-en-light', 'A second unrelated AppSpec is rendered by the same HTTP Runtime with generated loan inputs and return action.');
    await page.route(`${libraryHost}/v1/apps/app_library/commands`, async route => {
      const response = await route.fetch();
      assert.equal(response.status(), 200);
      await route.abort('failed');
    });
    await id(page, 'runtime-submit').click();
    await includes(page, 'runtime-status', 'outcome is unknown');
    await connection(page);
    await id(page, 'runtime-disconnect').click();
    assert.equal(await id(page, 'runtime-credential').inputValue(), '');
    assert.equal(await id(page, 'runtime-records').count(), 0);
    assert.equal(await id(page, 'runtime-server').isDisabled(), true, 'unresolved intent keeps its original application binding');
    await id(page, 'runtime-credential').fill(token);
    await id(page, 'runtime-recover').click();
    await includes(page, 'runtime-receipt', 'Server confirmed');
    await includes(page, 'runtime-records', 'returned');
    assert.equal(await id(page, 'runtime-submit').count(), 0);
  });
} finally {
  await restoreAccess();
  await browser.close();
}

const report = {
  source,
  boundary: 'actual local single-tenant HTTP hosts backed by libSQL; route interception delays/drops only transport responses after real host execution',
  passed: results.filter(result => result.passed).length,
  failed: results.filter(result => !result.passed).length,
  results, captures,
  notCovered: ['Turso remote service', 'CMP/native targets', 'real provider identity', 'cross-reload unresolved request recovery', 'screen reader'],
};
const reportPath = path.join(evidenceDir, 'runtime-browser-report.json');
await writeFile(reportPath, JSON.stringify(report, null, 2));
console.log(`${report.passed}/${results.length} scenarios passed; report: ${reportPath}`);
if (report.failed || !results.length) process.exitCode = 1;
