## MODIFIED Requirements

### Requirement: RssParser reuses HTTP client

The system SHALL create one `reqwest::Client` instance at `RssParser` construction time and reuse it for all subsequent feed fetches. The client SHALL be configured with `user_agent` and `timeout` from `ParserConfig`.

#### Scenario: Client created once at construction

- **WHEN** `RssParser::new(config)` is called
- **THEN** a single `reqwest::Client` SHALL be built with the configured user_agent and timeout
- **THEN** the client SHALL be stored as a field on `RssParser`

#### Scenario: Fetch reuses client instance

- **WHEN** `fetch_and_parse` is called multiple times
- **THEN** each call SHALL use `self.client.get(&source.url).send().await`
- **THEN** no new `reqwest::Client` SHALL be created per fetch
- **THEN** HTTP connection pooling SHALL work across multiple feed fetches
