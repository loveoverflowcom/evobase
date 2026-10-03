//! An explicitly simulated consumer of the canonical AppSpec policy/command definitions.
use crate::tr;
use evobase_appspec::policy::{
    ActorId, AuthorityError, BindingFacts, CommandDecision, Grant, HostAuthority, LinePatch,
    MembershipFacts, OwnerRolePolicy, PolicyError, ProjectionPurpose, RawSubmitOrder, ReceiptBook,
    RequestChannel, RoleId, SessionFacts, SessionVerifier, SubmitOrderRule, decide_submit_order,
    project,
};
use evobase_appspec::{
    CheckedAppSpec, CheckedRecords, RawAppSpec, RawRecord, RecordId, Scope, TableId, Value,
    fixtures,
};
use leptos::prelude::*;
use std::collections::BTreeSet;

/// Fixture adapter only. These selections do not establish identity outside this local simulation.
struct SimulatedVerifier {
    session: SessionFacts,
    membership: MembershipFacts,
    binding: BindingFacts,
}
impl SessionVerifier for SimulatedVerifier {
    fn verify_session(&self, credential: &str) -> Result<SessionFacts, AuthorityError> {
        if credential != "local_simulation" {
            return Err(AuthorityError::Unverified);
        }
        Ok(self.session.clone())
    }
    fn current_membership(
        &self,
        _: &ActorId,
        _: &Scope,
    ) -> Result<MembershipFacts, AuthorityError> {
        Ok(self.membership.clone())
    }
    fn registry_binding(&self, _: &Scope) -> Result<BindingFacts, AuthorityError> {
        Ok(self.binding.clone())
    }
}

fn simulated_host(
    scope: &Scope,
    actor: &str,
    role: &str,
    grants: BTreeSet<Grant>,
    active: bool,
) -> Result<HostAuthority<SimulatedVerifier>, PolicyError> {
    let actor = ActorId::new(actor)?;
    let roles = if role.is_empty() {
        BTreeSet::new()
    } else {
        [RoleId::new(role)?].into_iter().collect()
    };
    Ok(HostAuthority::new(SimulatedVerifier {
        session: SessionFacts {
            actor: actor.clone(),
            expires_at: 1000,
            revoked: false,
        },
        membership: MembershipFacts {
            actor,
            scope: scope.clone(),
            active,
            revision: 1,
            grants,
            roles,
        },
        binding: BindingFacts {
            scope: scope.clone(),
            binding_id: "local_simulation_binding".to_owned(),
            allowed_mutation_origins: BTreeSet::new(),
        },
    }))
}

fn checked(
    raw: RawAppSpec,
    rows: &[RawRecord],
    table: &TableId,
) -> Result<(CheckedAppSpec, Scope, CheckedRecords, OwnerRolePolicy), PolicyError> {
    let spec = CheckedAppSpec::compile(raw).map_err(|_| PolicyError::InvalidPolicy)?;
    let scope = Scope::new("tenant_local_fixture", spec.app_id().clone())
        .map_err(|_| PolicyError::ScopeMismatch)?;
    let facts = spec
        .validate_records(&scope, rows)
        .map_err(|_| PolicyError::InvalidInput)?;
    let rule = spec
        .definition()
        .policies
        .iter()
        .find(|rule| &rule.table_id == table)
        .ok_or(PolicyError::InvalidPolicy)?;
    let policy = OwnerRolePolicy::check(&spec, rule.clone())?;
    Ok((spec, scope, facts, policy))
}

#[derive(Clone)]
enum Notice {
    Ready,
    SourceAdded,
    SourceEdited,
    Applied(usize),
    Replay,
    Error(PolicyError),
}
fn message(vi: bool, notice: &Notice) -> String {
    match notice {
        Notice::Ready => tr(
            vi,
            "Chọn quyền và danh tính giả lập để kiểm tra quy tắc.",
            "Choose simulated identity and grants to inspect the rule.",
        ),
        Notice::SourceAdded => tr(
            vi,
            "Đã thêm quy tắc vào AppSpec của bản nháp.",
            "The rule was added to your draft AppSpec.",
        ),
        Notice::SourceEdited => tr(
            vi,
            "Quy tắc trong AppSpec đã cập nhật.",
            "The AppSpec rule was updated.",
        ),
        Notice::Applied(count) => format!(
            "{} {count} {}",
            tr(vi, "Đã mô phỏng", "Simulated"),
            tr(
                vi,
                "bản ghi đã kiểm tra; trạng thái submitted.",
                "checked writes; state submitted."
            )
        ),
        Notice::Replay => tr(
            vi,
            "Lặp lại cùng khóa và ý định: trả lại biên nhận mô phỏng.",
            "Same key and intent: replayed the simulated receipt.",
        ),
        Notice::Error(PolicyError::Authority(AuthorityError::Revoked)) => tr(
            vi,
            "Từ chối: quyền thành viên hiện tại đã bị thu hồi.",
            "Denied: current membership is revoked.",
        ),
        Notice::Error(PolicyError::Authority(AuthorityError::MissingGrant)) => tr(
            vi,
            "Từ chối: thiếu quyền nghiệp vụ hiện tại.",
            "Denied: current business grant is missing.",
        ),
        Notice::Error(PolicyError::Denied) => tr(
            vi,
            "Từ chối: quy tắc chủ sở hữu/vai trò không cho phép thao tác.",
            "Denied: the owner/role rule does not permit this action.",
        ),
        Notice::Error(PolicyError::InvalidPolicy) => tr(
            vi,
            "Chọn bảng Order và thêm quy tắc mẫu, hoặc sửa định nghĩa không được hỗ trợ.",
            "Select Order and add the example rule, or repair an unsupported definition.",
        ),
        Notice::Error(PolicyError::IntentConflict) => tr(
            vi,
            "Khóa đã dùng cho ý định khác. Giữ nguyên dữ liệu khi thử lại.",
            "The key belongs to a different intent. Preserve the input when retrying.",
        ),
        Notice::Error(PolicyError::InvalidQuantity) => tr(
            vi,
            "Số lượng phải là số nguyên từ 1 đến 10000.",
            "Quantity must be an integer from 1 to 10000.",
        ),
        Notice::Error(PolicyError::NotDraft) => tr(
            vi,
            "Đơn hàng phải ở trạng thái draft.",
            "The order must be in draft state.",
        ),
        Notice::Error(PolicyError::ImmutableField) => tr(
            vi,
            "Trường này được bảo vệ trong thao tác gửi đơn.",
            "This field is protected in the submit command.",
        ),
        Notice::Error(PolicyError::EmptyOrder) => tr(
            vi,
            "Đơn hàng cần ít nhất một dòng dữ liệu.",
            "The order needs at least one line.",
        ),
        Notice::Error(PolicyError::ScopeMismatch) => tr(
            vi,
            "Dữ liệu mô phỏng không khớp phạm vi ứng dụng.",
            "The simulated facts do not match the app scope.",
        ),
        Notice::Error(PolicyError::Overflow) => tr(
            vi,
            "Giá trị vượt giới hạn. Quy tắc hiện tại được giữ nguyên.",
            "The value exceeds the supported limit. The current rule was preserved.",
        ),
        Notice::Error(PolicyError::InvalidInput) => tr(
            vi,
            "Dữ liệu không hợp lệ. Kiểm tra ghi chú, số lượng và khóa thử lại.",
            "Invalid input. Check the notes, quantity and retry key.",
        ),
        Notice::Error(PolicyError::Authority(_)) => tr(
            vi,
            "Danh tính hoặc phạm vi hiện tại không được phép.",
            "The current identity or scope is not authorized.",
        ),
    }
}

fn grant_label(grant: Grant, vi: bool) -> String {
    match grant {
        Grant::Design => tr(vi, "Thiết kế", "Design"),
        Grant::Publish => tr(vi, "Phát hành", "Publish"),
        Grant::Manage => tr(vi, "Quản lý", "Manage"),
        Grant::Read => tr(vi, "Đọc", "Read"),
        Grant::Write => tr(vi, "Ghi", "Write"),
        Grant::Submit => tr(vi, "Gửi đơn", "Submit"),
    }
}

fn localized_sentence(raw: &RawAppSpec, table: &TableId, vi: bool) -> String {
    let Some(rule) = raw.policies.iter().find(|rule| &rule.table_id == table) else {
        return tr(vi, "Chưa có quy tắc cho bảng này", "No rule for this table");
    };
    let owner = raw
        .tables
        .iter()
        .find(|table| table.id == rule.table_id)
        .and_then(|table| {
            table
                .fields
                .iter()
                .find(|field| field.id == rule.owner_field)
        })
        .map(|field| field.name.as_str())
        .unwrap_or(rule.owner_field.as_str());
    let roles = rule
        .read_roles
        .iter()
        .map(RoleId::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    if vi {
        format!(
            "{}: đọc khi {owner} là danh tính hiện tại hoặc vai trò hiện tại thuộc [{roles}]",
            rule.rule_id
        )
    } else {
        format!(
            "{}: read when {owner} equals the current actor or the current role is in [{roles}]",
            rule.rule_id
        )
    }
}

#[component]
pub fn PolicyInspector(
    spec: RwSignal<RawAppSpec>,
    rows: RwSignal<Vec<RawRecord>>,
    table_id: RwSignal<TableId>,
    locale: RwSignal<bool>,
    #[prop(optional)] saving: Option<RwSignal<bool>>,
    #[prop(optional)] paused: Option<RwSignal<bool>>,
    #[prop(optional)] on_source_change: Option<Callback<()>>,
) -> impl IntoView {
    let sandbox = RwSignal::new(rows.get_untracked());
    let revision = RwSignal::new(1u64);
    let receipts = StoredValue::new(ReceiptBook::default());
    let actor = RwSignal::new("actor_alice".to_owned());
    let role = RwSignal::new(String::new());
    let grants = RwSignal::new(
        [Grant::Read, Grant::Submit]
            .into_iter()
            .collect::<BTreeSet<_>>(),
    );
    let active = RwSignal::new(true);
    let notes = RwSignal::new("  urgent  ".to_owned());
    let quantity = RwSignal::new("3".to_owned());
    let key = RwSignal::new("submit-1".to_owned());
    let notice = RwSignal::new(Notice::Ready);
    let editing_disabled = move || {
        saving.is_some_and(|signal| signal.get()) || paused.is_some_and(|signal| signal.get())
    };

    let add_source = move |_| {
        if editing_disabled() {
            return;
        }
        let mut definition = spec.get_untracked();
        let example = fixtures::example_policy_spec();
        let selected = table_id.get_untracked();
        let Some(rule) = example
            .policies
            .into_iter()
            .find(|rule| rule.table_id == selected)
        else {
            notice.set(Notice::Error(PolicyError::InvalidPolicy));
            return;
        };
        if definition
            .policies
            .iter()
            .any(|existing| existing.table_id == selected)
        {
            return;
        }
        let rule_id = rule.rule_id.clone();
        definition.policies.push(rule);
        definition.submit_rules.extend(
            example
                .submit_rules
                .into_iter()
                .filter(|rule| rule.policy_rule_id == rule_id),
        );
        match CheckedAppSpec::compile(definition.clone()) {
            Ok(_) => {
                spec.set(definition);
                if let Some(callback) = on_source_change {
                    callback.run(());
                }
                notice.set(Notice::SourceAdded);
            }
            Err(_) => notice.set(Notice::Error(PolicyError::InvalidPolicy)),
        }
    };
    let edit_role = move |submit: bool| {
        if editing_disabled() {
            return;
        }
        let mut definition = spec.get_untracked();
        let Some(rule) = definition
            .policies
            .iter_mut()
            .find(|rule| rule.table_id == table_id.get_untracked())
        else {
            return;
        };
        let allowed = if submit {
            &mut rule.submit_roles
        } else {
            &mut rule.read_roles
        };
        let toggled = RoleId::new(if submit {
            "role_operator"
        } else {
            "role_auditor"
        })
        .expect("fixed simulated role");
        if !allowed.remove(&toggled) {
            allowed.insert(toggled);
        }
        let Some(next_revision) = rule.revision.checked_add(1) else {
            notice.set(Notice::Error(PolicyError::Overflow));
            return;
        };
        rule.revision = next_revision;
        match CheckedAppSpec::compile(definition.clone()) {
            Ok(_) => {
                spec.set(definition);
                if let Some(callback) = on_source_change {
                    callback.run(());
                }
                notice.set(Notice::SourceEdited);
            }
            Err(_) => notice.set(Notice::Error(PolicyError::InvalidPolicy)),
        }
    };
    let run_command = move |_| {
        let result = (|| -> Result<CommandDecision, PolicyError> {
            let (definition, scope, facts, policy) = checked(
                spec.get_untracked(),
                &sandbox.get_untracked(),
                &table_id.get_untracked(),
            )?;
            let raw_command = definition
                .definition()
                .submit_rules
                .iter()
                .find(|rule| rule.policy_rule_id == policy.rule_id())
                .ok_or(PolicyError::InvalidPolicy)?;
            let command = SubmitOrderRule::check(&definition, &policy, raw_command.clone())?;
            let host = simulated_host(
                &scope,
                &actor.get_untracked(),
                &role.get_untracked(),
                grants.get_untracked(),
                active.get_untracked(),
            )?;
            let quantity = quantity
                .get_untracked()
                .parse::<i64>()
                .map_err(|_| PolicyError::InvalidQuantity)?;
            let order_id = RecordId::new("rec_order_1").map_err(|_| PolicyError::InvalidInput)?;
            let line_patches = facts.records().iter().filter(|row| row.table_id() == &raw_command.line_table
                && matches!(row.values().get(&raw_command.line_order_field), Some(Value::Ref(reference)) if reference.record_id == order_id))
                .map(|row| LinePatch { line_id: row.id().clone(), changes: [(raw_command.quantity_field.clone(), Value::Integer(quantity))].into_iter().collect() }).collect();
            let request = RawSubmitOrder {
                order_id,
                idempotency_key: key.get_untracked(),
                changes: [(
                    raw_command.notes_field.clone(),
                    Value::Text(notes.get_untracked()),
                )]
                .into_iter()
                .collect(),
                line_patches,
            };
            receipts.with_value(|receipts| {
                decide_submit_order(
                    &host,
                    "local_simulation",
                    &scope,
                    &RequestChannel::NativeBearer,
                    100,
                    &definition,
                    &facts,
                    revision.get_untracked(),
                    &policy,
                    &command,
                    receipts,
                    &request,
                )
            })
        })();
        match result {
            Ok(CommandDecision::Apply(batch)) => {
                let count = batch.writes().len();
                match receipts.try_update_value(|receipts| receipts.record_applied(&batch)) {
                    Some(Ok(())) => {}
                    Some(Err(error)) => {
                        notice.set(Notice::Error(error));
                        return;
                    }
                    None => {
                        notice.set(Notice::Error(PolicyError::InvalidInput));
                        return;
                    }
                }
                // This is only the in-memory simulation adapter, not an atomic persistence claim.
                sandbox.update(|rows| {
                    for write in batch.writes() {
                        if let Some(row) = rows
                            .iter_mut()
                            .find(|row| row.table_id == write.table_id && row.id == write.id)
                        {
                            *row = write.clone();
                        }
                    }
                });
                revision.update(|revision| *revision += 1);
                notice.set(Notice::Applied(count));
            }
            Ok(CommandDecision::Replay(_)) => notice.set(Notice::Replay),
            Err(error) => notice.set(Notice::Error(error)),
        }
    };
    let projection = move || {
        let (_, scope, facts, policy) = checked(spec.get(), &sandbox.get(), &table_id.get())?;
        let host = simulated_host(
            &scope,
            &actor.get(),
            &role.get(),
            grants.get(),
            active.get(),
        )?;
        project(
            &host,
            "local_simulation",
            &scope,
            &RequestChannel::NativeBearer,
            100,
            &facts,
            &policy,
            ProjectionPurpose::Query,
        )
        .map(|output| {
            output
                .rows()
                .iter()
                .map(|row| row.id().as_str().to_owned())
                .collect::<Vec<_>>()
        })
    };
    view! {
        <section class="surface policy-panel" data-testid="policy-panel">
            <h2>{move || tr(locale.get(), "Quy tắc & thao tác", "Policies & commands")}</h2>
            <p class="support">{move || tr(locale.get(), "Mô phỏng cục bộ · danh tính và quyền giả lập. Đặt lại để sao chép dữ liệu bản nháp hiện tại vào bộ nhớ mô phỏng.", "Local simulation · fixture identity and grants. Reset copies the current draft facts into the simulation's memory.")}</p>
            <button data-testid="policy-add-source" disabled=editing_disabled on:click=add_source>{move || tr(locale.get(), "Thêm quy tắc mẫu cho Order", "Add example Order rule")}</button>
            <p data-testid="policy-source">{move || spec.get().policies.iter().find(|rule| rule.table_id == table_id.get()).map(|rule| format!("{} · v{}", rule.rule_id, rule.revision)).unwrap_or_else(|| tr(locale.get(), "Chưa có quy tắc cho bảng này", "No rule for this table"))}</p>
            <p data-testid="policy-sentence">{move || localized_sentence(&spec.get(), &table_id.get(), locale.get())}</p>
            <fieldset><legend>{move || tr(locale.get(), "Vai trò được quy tắc cho phép", "Roles allowed by the rule")}</legend>
                <label><input data-testid="policy-allow-auditor" disabled=move || editing_disabled() || !spec.get().policies.iter().any(|rule| rule.table_id == table_id.get()) type="checkbox" prop:checked=move || spec.get().policies.iter().find(|rule| rule.table_id == table_id.get()).is_some_and(|rule| rule.read_roles.contains(&RoleId::new("role_auditor").expect("fixed role"))) on:change=move |_| edit_role(false)/>{move || tr(locale.get(), "Auditor được đọc", "Auditor may read")}</label>
                <label><input data-testid="policy-allow-operator" disabled=move || editing_disabled() || !spec.get().policies.iter().any(|rule| rule.table_id == table_id.get()) type="checkbox" prop:checked=move || spec.get().policies.iter().find(|rule| rule.table_id == table_id.get()).is_some_and(|rule| rule.submit_roles.contains(&RoleId::new("role_operator").expect("fixed role"))) on:change=move |_| edit_role(true)/>{move || tr(locale.get(), "Operator được gửi đơn", "Operator may submit")}</label>
            </fieldset>
            <label>{move || tr(locale.get(), "Danh tính giả lập", "Simulated actor")}<select data-testid="policy-actor" prop:value=move || actor.get() on:change=move |event| actor.set(event_target_value(&event))><option value="actor_alice">"Alice"</option><option value="actor_bob">"Bob"</option></select></label>
            <label>{move || tr(locale.get(), "Vai trò hiện tại giả lập", "Simulated current role")}<select data-testid="policy-role" prop:value=move || role.get() on:change=move |event| role.set(event_target_value(&event))><option value="">{move || tr(locale.get(), "Không có", "None")}</option><option value="role_auditor">"Auditor"</option><option value="role_operator">"Operator"</option></select></label>
            <fieldset><legend>{move || tr(locale.get(), "Quyền host độc lập (giả lập)", "Independent host grants (simulated)")}</legend>
                {[Grant::Design, Grant::Publish, Grant::Manage, Grant::Read, Grant::Write, Grant::Submit].into_iter().map(|grant| view! { <label><input type="checkbox" data-testid=format!("policy-grant-{grant:?}") prop:checked=move || grants.get().contains(&grant) on:change=move |_| grants.update(|grants| { if !grants.remove(&grant) { grants.insert(grant); } })/>{move || grant_label(grant, locale.get())}</label> }).collect_view()}
            </fieldset>
            <label><input data-testid="policy-active" type="checkbox" prop:checked=move || active.get() on:change=move |_| active.update(|active| *active = !*active)/>{move || tr(locale.get(), "Thành viên hiện tại còn hiệu lực", "Current membership is active")}</label>
            <p data-testid="policy-visible" aria-live="polite">{move || match projection() { Ok(ids) => format!("{} {} [{}]", tr(locale.get(), "Bản ghi được phép đọc:", "Readable records:"), ids.len(), ids.join(", ")), Err(error) => message(locale.get(), &Notice::Error(error)) }}</p>
            <label>{move || tr(locale.get(), "Ghi chú gửi đơn", "Submit notes")}<input data-testid="policy-notes" prop:value=move || notes.get() on:input=move |event| notes.set(event_target_value(&event))/></label>
            <label>{move || tr(locale.get(), "Số lượng dòng đơn", "Order line quantity")}<input data-testid="policy-quantity" inputmode="numeric" prop:value=move || quantity.get() on:input=move |event| quantity.set(event_target_value(&event))/></label>
            <label>{move || tr(locale.get(), "Khóa thử lại", "Retry key")}<input data-testid="policy-key" prop:value=move || key.get() on:input=move |event| key.set(event_target_value(&event))/></label>
            <div class="actions"><button data-testid="policy-submit" on:click=run_command>{move || tr(locale.get(), "Mô phỏng SubmitOrder", "Simulate SubmitOrder")}</button><button data-testid="policy-reset" on:click=move |_| { sandbox.set(rows.get_untracked()); receipts.set_value(ReceiptBook::default()); revision.set(1); notice.set(Notice::Ready); }>{move || tr(locale.get(), "Đặt lại mô phỏng", "Reset simulation")}</button></div>
            <p data-testid="policy-decision" role="status" aria-live="polite">{move || message(locale.get(), &notice.get())}</p>
            <details><summary>{move || tr(locale.get(), "Phạm vi hỗ trợ", "Supported subset")}</summary><p>{move || tr(locale.get(), "Chủ sở hữu hoặc vai trò hiện tại; quyền host được kiểm tra trước biên nhận. Ref, join, SQL/RLS và truy vấn toàn bảng bị hạn chế nằm ngoài phạm vi này.", "Owner or current role; host grants checked before receipts. Ref output, joins, SQL/RLS and restricted complete scans are outside this subset.")}</p></details>
        </section>
    }
}
