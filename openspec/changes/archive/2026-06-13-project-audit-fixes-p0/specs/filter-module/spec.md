## MODIFIED Requirements

### Requirement: Hot event upsert uses ON CONFLICT

The system SHALL upsert hot_event records using SQLite's `ON CONFLICT(keyword_id, hour_bucket) DO UPDATE` syntax to preserve the row's `id` and maintain foreign key integrity with `push_records`. The upsert and push_records insert SHALL be wrapped in a single database transaction.

#### Scenario: First hotspot for keyword in hour — insert

- **WHEN** a hotspot is detected and no existing row matches the (keyword_id, hour_bucket) pair
- **THEN** the system SHALL INSERT a new `hot_events` row within a transaction
- **THEN** push_records SHALL be created referencing the new row's id within the same transaction
- **THEN** the transaction SHALL COMMIT before marking articles as processed

#### Scenario: Repeat detection in same hour — update

- **WHEN** a hotspot is re-detected for the same (keyword_id, hour_bucket) pair
- **THEN** the system SHALL UPDATE the existing row's `count`, `mean_historical`, and `stddev_historical`
- **THEN** the row's `id` SHALL NOT change
- **THEN** existing push_records referencing this hot_event SHALL remain valid

#### Scenario: Transaction failure rolls back

- **WHEN** push_records insert fails within the transaction (e.g., FK violation)
- **THEN** the transaction SHALL ROLLBACK
- **THEN** hot_event changes SHALL be reverted
- **THEN** articles SHALL NOT be marked as processed
