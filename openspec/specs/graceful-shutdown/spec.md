# graceful-shutdown

## Purpose

Coordinated graceful shutdown of all background tasks (Parser, Filter, Pusher) and the axum HTTP server using `tokio_util::sync::CancellationToken`, triggered by Ctrl+C (SIGINT).

## Requirements

### Requirement: CancellationToken-based graceful shutdown

The system SHALL use `tokio_util::sync::CancellationToken` to coordinate graceful shutdown of all background tasks when Ctrl+C is received.

#### Scenario: Ctrl+C triggers cancellation

- **WHEN** the process receives Ctrl+C (SIGINT)
- **THEN** the system SHALL call `cancel.cancel()` to notify all background tasks
- **AND** each background task SHALL log a "shutting down gracefully" message and exit its loop

#### Scenario: Background task exits on cancellation

- **WHEN** a background task (Parser/Filter/Pusher) is running and `cancel.cancelled()` resolves
- **THEN** the task SHALL break out of its main loop and allow the spawned future to complete

#### Scenario: Axum server shuts down gracefully

- **WHEN** the cancellation token is triggered
- **THEN** the axum server SHALL stop accepting new connections via `with_graceful_shutdown`
- **AND** SHALL complete in-flight requests before shutting down

#### Scenario: Main function waits for all tasks

- **WHEN** the cancellation token is triggered
- **THEN** the `main` function SHALL not exit until the axum server graceful shutdown completes
- **AND** the tokio runtime SHALL wait for all spawned tasks to finish before the process exits
