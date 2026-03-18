# Eonbase PlantUML Diagrams

Bo diagram nay duoc viet de ho tro 2 nhu cau:

- Tai lieu kien truc cho team
- Doc nhanh flow cua `eonbase` ma khong can mo tung crate

## Danh sach diagram

- `01_component_overview.puml`: Tong quan cac crate va phu thuoc
- `02_auth_rls_sequence.puml`: Request flow voi JWT va RLS trong Postgres
- `03_messaging_sse_flow.puml`: Dong chay event SSE va fan-out theo user
- `04_sse_connection_state.puml`: Vong doi ket noi SSE trong memory hub
- `05_bootstrap_sequence.puml`: Trinh tu startup cua server app
- `06_runtime_deployment.puml`: Topology runtime cua process va PostgreSQL
- `07_core_contracts_class.puml`: Traits o `eonbase-core` va concrete implementations
- `08_auth_token_lifecycle.puml`: Login, refresh, va cach 3 loai token duoc su dung
- `09_rest_gateway_flow.puml`: Flow PostgREST-like khi goi `/rest/:table`
- `10_data_model_erd.puml`: Data model MVP va boundary RLS

## Cach su dung

- Diagram `01`, `06`, `07` hop voi phan "Architecture"
- Diagram `02`, `03`, `05`, `08`, `09` hop voi phan "Request Lifecycle"
- Diagram `04`, `10` hop voi phan "Operational Notes" va "Database"

## Ghi chu

- Cac ten module duoc dat sat voi code hien tai trong repo.
- Cac sequence diagram phan anh implementation thuc te:
  - JWT access token duoc validate o middleware
  - Notification token duoc dung cho `/events`
  - Postgres nhan `request.jwt.claim.sub` qua `set_config(...)`
