use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DemoDomainIr {
    pub package: &'static str,
    pub refinements: Vec<&'static str>,
    pub entities: Vec<EntityIr>,
    pub workflows: Vec<WorkflowIr>,
    pub commands: Vec<CommandIr>,
    pub events: Vec<&'static str>,
    pub policies: Vec<&'static str>,
    pub generators: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityIr {
    pub name: &'static str,
    pub fields: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowIr {
    pub name: &'static str,
    pub states: Vec<&'static str>,
    pub transitions: Vec<TransitionIr>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransitionIr {
    pub name: &'static str,
    pub from: &'static str,
    pub to: &'static str,
    pub event: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandIr {
    pub name: &'static str,
    pub policy: &'static str,
}

pub fn demo_domain_ir() -> DemoDomainIr {
    DemoDomainIr {
        package: "evobase.demo.commerce",
        refinements: vec![
            "Email",
            "NonEmptyString",
            "PositiveInt",
            "Money",
            "UserId",
            "VerifiedUserId",
            "OrderId",
        ],
        entities: vec![
            EntityIr {
                name: "User",
                fields: vec!["id: UserId", "email: Email", "verified: bool"],
            },
            EntityIr {
                name: "Order",
                fields: vec![
                    "id: OrderId",
                    "buyer_id: UserId",
                    "total: Money",
                    "status: OrderStatus",
                    "version: i64",
                ],
            },
        ],
        workflows: vec![WorkflowIr {
            name: "OrderLifecycle",
            states: vec!["Draft", "Paid", "Shipped", "Cancelled"],
            transitions: vec![
                TransitionIr {
                    name: "pay",
                    from: "Draft",
                    to: "Paid",
                    event: "OrderPaid",
                },
                TransitionIr {
                    name: "ship",
                    from: "Paid",
                    to: "Shipped",
                    event: "OrderShipped",
                },
                TransitionIr {
                    name: "cancel_draft",
                    from: "Draft",
                    to: "Cancelled",
                    event: "OrderCancelled",
                },
                TransitionIr {
                    name: "cancel_paid",
                    from: "Paid",
                    to: "Cancelled",
                    event: "OrderCancelled",
                },
            ],
        }],
        commands: vec![
            CommandIr {
                name: "CreateOrder",
                policy: "actor is verified buyer",
            },
            CommandIr {
                name: "PayOrder",
                policy: "actor is verified buyer and has pay_order",
            },
            CommandIr {
                name: "ShipOrder",
                policy: "actor has ship_order",
            },
            CommandIr {
                name: "CancelOrder",
                policy: "actor has cancel_order or owns order",
            },
        ],
        events: vec![
            "UserRegistered",
            "EmailVerified",
            "OrderDrafted",
            "OrderPaid",
            "OrderShipped",
            "OrderCancelled",
        ],
        policies: vec![
            "verified_actor",
            "buyer_owns_order",
            "capability:pay_order",
            "capability:ship_order",
            "capability:cancel_order",
        ],
        generators: vec![
            "postgres schema",
            "REST command API",
            "domain event outbox",
            "documentation metadata",
        ],
    }
}
