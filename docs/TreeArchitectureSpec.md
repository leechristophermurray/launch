# Tree Architecture Specification
Version: 1.0.0

## 1. Introduction
Tree Architecture is a strict, unidirectional architectural pattern designed to enforce separation of concerns, testability, and stability in software projects. It visualizes dependencies as a "tree" where changes in the leaves (Infrastructure) do not affect the roots (Domain).

## 2. The Dependency Rule
**The Golden Rule:** Dependencies can ONLY point downwards or deeper into the center.
*   **Infrastructure** -> **Interface** / **Application** / **Domain**
*   **Interface** -> **Application** / **Domain**
*   **Application** -> **Domain**
*   **Domain** -> **Nothing** (Pure)

Cycles are strictly forbidden. Side-dependencies (sibling modules) are permitted only if they do not create a cycle.

## 3. Layer Definitions & Colors

### 🔴 Domain Layer (Core)
*   **Color**: Red
*   **Responsibility**: The heart of the software. Contains pure business logic, entities, value objects, and domain services.
*   **Dependencies**: None. Pure standard library only.
*   **Strict Rules**:
    *   Must NOT depend on `Application`, `Interface`, or `Infrastructure`.
    *   Entities should be plain objects (POJOs/structs).

### 🟡 Application Layer (Orchestration)
*   **Color**: Yellow
*   **Responsibility**: Orchestrates domain objects to fulfill use cases. Handles application logic but relies on the Domain for business rules.
*   **Dependencies**: `Domain`.
*   **Strict Rules**:
    *   Must defines Port definitions (Traits/Interfaces) if they are specific to application needs (e.g. `IFileSystem`).

### 🟢 Interface Layer (Ports/Gateways)
*   **Color**: Green
*   **Responsibility**: Defines the contracts (ports) for external interaction. This layer effectively acts as the "API Surface" of your core logic.
*   **Dependencies**: `Application`, `Domain`.
*   **Strict Rules**:
    *   **Naming Convention**: All traits/interfaces MUST start with `I` (e.g., `IUserRepository`, `IEmailService`).
    *   Contains NO implementation logic, only signatures.

### 🔵 Infrastructure Layer (Adapters)
*   **Color**: Blue
*   **Responsibility**: Implements the interfaces defined in the Interface/Application layers. Interacts with the "real world" (Database, File System, Network, UI).
*   **Dependencies**: `Interface`, `Application`, `Domain`.
*   **Strict Rules**:
    *   **Naming Convention**: All concrete implementation structs/classes MUST end with `Adapter` (e.g., `PostgresUserAdapter`, `RealFileSystemAdapter`).
    *   This is the "Dirty" layer. All side effects live here.

## 4. Directory Structure
A compliant project MUST follow this directory structure. Abbreviations are NOT allowed (e.g., use `infrastructure/`, not `infra/`).

```text
src/
├── domain/            # 🔴 Pure Business Logic
│   ├── model.rs
│   └── rules.rs
├── application/       # 🟡 Use Cases & Orchestration
│   ├── use_cases/
│   └── ports/         # (Optional) Application-specific ports
├── interface/         # 🟢 Ports & Contracts
│   ├── api/
│   └── repositories/  # IRepository definitions
├── infrastructure/    # 🔵 Adapters & Implementations
│   ├── database/
│   │   └── postgres_adapter.rs
│   └── filesystem/
│       └── s3_adapter.rs
└── main.rs            # ⚪ Composition Root (Wiring)
```

## 5. Compliance & Validation
The `tacli` tool enforces these rules strictly.

### Violations
1.  **Dependency Violation**: A layer depending on a layer above it (e.g., Domain importing Infrastructure).
2.  **Naming Violation**:
    *   Infrastructure struct without `Adapter` suffix.
    *   Interface trait without `I` prefix.
3.  **Configuration Violation**: Ambiguous folder names (e.g., having both `infra/` and `infrastructure/`).

### Verification
Run the following command to verify compliance:
```bash
tacli check .
```
This will report any architectural or naming violations and calculate the "Blast Radius" of potential changes.
