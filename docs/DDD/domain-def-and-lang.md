# Cortex — Domain Definition & Ubiquitous Language

> **Product:** Cortex
> **Company:** nullbrain
> **Version:** 0.2.0 (Open Questions Resolved)
> **Date:** 2026-02-15

---

## 1. Vision Statement

Cortex is a local-first knowledge management tool for desktop, mobile, and web. It combines the structured data capabilities of Notion with the linked, file-based philosophy of Obsidian — powered by LLM intelligence. All data lives on the user's device first, with optional premium cloud sync and AI services.

---

## 2. Core Domain

**Knowledge Management** — the act of capturing, organizing, retrieving, and interacting with personal and professional knowledge.

### Core Activities (User Verbs)

| Activity | Description |
|----------|-------------|
| **Author** | Create and edit written documents using a markdown-based block editor |
| **Store** | Import and manage files (photos, PDFs, CSVs, etc.) as first-class knowledge items |
| **Retrieve** | Search knowledge via full-text search, semantic/vector search, or conversational LLM interaction |

---

## 3. Ubiquitous Language (Glossary)

### Organizational Concepts

| Term | Definition |
|------|------------|
| **Space** | The top-level boundary of knowledge in Cortex. A Space is a self-contained, user-accessible directory on the local filesystem. A user may have multiple Spaces (e.g., "Work", "Personal"). Each Space owns its own organizational hierarchy, encryption settings, and sync configuration. Analogous to a library. Spaces are isolated — items cannot reference across Space boundaries. |
| **Folder** | A nestable container within a Space (or another Folder) used to organize items. Supports unlimited nesting depth. Each Folder maps to a directory on the filesystem. Each Folder is its own aggregate — it can be created, moved, or deleted independently without loading the entire Space tree. |
| **Item** | The base abstraction for anything stored in Cortex. Every piece of content — whether authored or imported — is an Item. Items are first-class citizens regardless of type. |
| **Tag** | A user-defined or system-suggested label attached to an item for cross-cutting categorization and discovery. Tags exist alongside the hierarchy and serve a different purpose: hierarchy is for *organization*, tags are for *discovery and intelligence*. |

### Item Types

| Term | Definition |
|------|------------|
| **Document** | An item authored and edited within Cortex using the markdown-based block editor (TipTap). Persisted as a `.md` file on disk. Supports custom nodes beyond standard markdown. |
| **Table** | A structured, Notion-style collection of records with typed columns (e.g., a reading list with title, author, rating, status). Stored as structured data in SQLite with a `.json` file exported to the Space's filesystem for portability. |
| **Attachment** | An imported file (photo, PDF, CSV, or any other binary/text file) stored in the Space's filesystem. Attachments are indexed in SQLite for metadata, search, and tagging but are not directly editable within Cortex's editor. |

### Content & Editing Concepts

| Term | Definition |
|------|------------|
| **Block** | A discrete unit of content within a document (paragraph, heading, list, image embed, table embed, etc.). Cortex uses TipTap's block-based model with custom node extensions. |
| **Node** | A custom TipTap extension that defines a new block type beyond standard markdown (e.g., callouts, embedded tables, synced blocks). |
| **Version** | A historical snapshot of an item's state. Documents maintain version history using incremental diffs, enabling rollback and diff comparison while minimizing storage overhead. |
| **Template** | A reusable starting point for creating new documents or tables with predefined structure, content, and metadata. |

### Storage & Sync Concepts

| Term | Definition |
|------|------------|
| **Index** | The SQLite database that serves as the source of truth for all metadata, relationships, tags, and structural data. Content files (markdown, attachments) are referenced by the index but stored on the filesystem. |
| **Sync Service** | A nullbrain-hosted cloud service (premium feature) that acts as the central coordination point for cross-device synchronization. Not required for single-device use. Self-hosting is not supported at launch. |
| **Sync Set** | The user-defined subset of a Space's content selected for synchronization to the Sync Service. Users choose what syncs — not everything must. |
| **Conflict** | A state where the same item has been modified on two or more devices while offline. Cortex surfaces conflicts for manual merge — it does not silently resolve them. |

### LLM & Intelligence Concepts

| Term | Definition |
|------|------------|
| **Provider** | An LLM backend configured by the user. Cortex is provider-agnostic. Supported provider types: local (Ollama, LM Studio), cloud API (OpenAI, Anthropic, etc. via user-supplied credentials), and nullbrain-hosted (open-source models hosted by nullbrain as a paid service). |
| **Knowledge Chat** | A conversational, multi-turn interface where the user asks questions and an LLM agent retrieves and reasons over the user's knowledge base (RAG-style). |
| **Librarian** | An autonomous LLM agent that analyzes a Space and proposes organizational actions (re-filing, tagging, linking). Supports three permission modes: per-action approval (default), batch approval (review a queue of proposals), and automated rules (user-defined standing permissions like "always auto-tag new items"). |
| **Embedding** | A vector representation of an item's content, stored in LanceDB and used for semantic search and retrieval. Generated per-item and updated on content change. |
| **Embedding Store** | A LanceDB instance local to each Space that stores vector embeddings for semantic search. LanceDB is embedded (no separate server process), columnar, and supports efficient approximate nearest neighbor search. |
| **Knowledge Query** | A retrieval operation that combines full-text search (via SQLite FTS) and semantic/vector search (via LanceDB) to find relevant items in a Space. Used by both the user's manual search and the LLM agents. |

### User & Access Concepts

| Term | Definition |
|------|------------|
| **Owner** | The single user who owns a Space. Cortex is single-user at launch. |
| **Collaborator** | (Future) A user granted access to a shared Space or subset of a Space. Collaboration is a future capability, not part of the initial domain. |

### Security Concepts

| Term | Definition |
|------|------------|
| **Space Encryption** | Optional at-rest encryption of a Space's contents on disk. User-configurable per Space. Uses per-file encryption to allow selective sync and granular access. Key derived from a user-provided passphrase. |

### Platform & Extensibility Concepts

| Term | Definition |
|------|------------|
| **API** | A local (and optionally network-exposed) interface that allows external tools, scripts, and plugins to interact with Cortex programmatically. |

---

## 4. Bounded Contexts

Each context owns its own models and rules. Communication between contexts happens through well-defined interfaces.

### 4.1 Authoring Context

**Responsibility:** Creating, editing, and versioning documents and tables.

**Key Aggregates:**
- Document (root: Document, children: Blocks, Versions)
- Table (root: Table, children: Columns, Records)
- Template

**Key Domain Events:**
- `DocumentCreated`, `DocumentUpdated`, `DocumentVersionSaved`
- `TableCreated`, `RecordAdded`, `RecordUpdated`

### 4.2 Organization Context

**Responsibility:** The Space's structure — folders, items, tags, and their relationships.

**Key Aggregates:**
- Space (root: Space — lightweight; does not contain the full folder tree)
- Folder (own aggregate — independently loadable and mutable)
- Tag

**Key Domain Events:**
- `ItemFiled`, `ItemMoved`, `ItemTagged`, `ItemUntagged`
- `FolderCreated`, `FolderMoved`, `FolderDeleted`
- `SpaceCreated`, `SpaceArchived`

### 4.3 Storage Context

**Responsibility:** Persistence of content to the filesystem and SQLite index. Managing attachments and binary files. Each Space has its own Index and FileStore.

**Key Aggregates:**
- Index (the SQLite metadata store)
- FileStore (the filesystem content store)

**Key Domain Events:**
- `AttachmentImported`, `FileWritten`, `IndexUpdated`

### 4.4 Intelligence Context

**Responsibility:** LLM integration, embeddings, search, knowledge chat, and the librarian agent. Operates within the scope of a single Space.

**Key Aggregates:**
- Provider (configured LLM backend)
- KnowledgeChat (a conversation session with retrieval)
- Librarian (the autonomous organizational agent)
- EmbeddingStore (LanceDB vector index, per-Space)

**Key Domain Events:**
- `EmbeddingGenerated`, `KnowledgeQueried`
- `LibrarianProposalCreated`, `LibrarianActionApproved`, `LibrarianActionRejected`, `LibrarianRuleCreated`
- `ChatMessageSent`, `ChatResponseGenerated`

### 4.5 Sync Context

**Responsibility:** Cross-device synchronization via the nullbrain Sync Service (premium). Conflict detection and manual merge.

**Key Aggregates:**
- SyncConnection (the authenticated connection to the nullbrain Sync Service)
- SyncSet (the user-defined subset of a Space's content selected for sync)
- Conflict (a detected divergence requiring manual resolution)

**Key Domain Events:**
- `SyncStarted`, `SyncCompleted`, `ConflictDetected`, `ConflictResolved`

---

## 5. Explicit Non-Goals

| Non-Goal | Notes |
|----------|-------|
| **Code editing / IDE** | Cortex is not a development environment. Code snippets in documents are display-only. |
| **Messaging / Chat (between users)** | Knowledge Chat is human-to-LLM, not human-to-human. |
| **Project Management** | No kanban boards, sprint planning, or workflow automation. Users may track things in tables, but Cortex doesn't model PM concepts. |
| **Calendar / Scheduling** | No date-based views, reminders, or scheduling. |
| **Web Clipping** | Not in scope for initial release. May be revisited. |
| **Self-hosted Sync** | Sync is a nullbrain-hosted premium service only. No self-hosted sync server at launch. |
| **Cross-Space References** | Items cannot link across Space boundaries. Spaces are fully isolated. |

---

## 6. Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| Local-first, 100% offline capable | Core product principle. All features must work without network. |
| SQLite as metadata index + filesystem for content | SQLite is source of truth for structure/metadata. Markdown and binaries live on disk in a user-accessible Space directory. |
| LanceDB for vector/embedding storage | Embedded (no server process), columnar, local-first. One LanceDB instance per Space. |
| Space mirrors organizational hierarchy on disk | Users can browse and interact with their Space outside the app (like Obsidian). |
| Folder as own aggregate | Folders are independently loadable and mutable. No need to load the full Space tree for folder operations. Enables efficient deep nesting. |
| Provider-agnostic LLM layer | Users bring their own credentials or use local models. No vendor lock-in. |
| Manual conflict resolution on sync | Respects the user's intent. No silent data loss. |
| Delta-based sync with manual conflict resolution | Simpler than CRDTs/OT. Track change timestamps and sync deltas to the nullbrain Sync Service. When divergence is detected, surface a conflict for the user to resolve manually. Appropriate given single-user-per-device and the premium sync model. |
| Open source + paid cloud features | Core app is free and open source. Sync Service and LLM hosting are paid services under nullbrain. |
| TipTap editor with markdown persistence | Proven editor framework. Markdown on disk ensures portability. Custom nodes extend beyond standard markdown. |
| Incremental diffs for version history | Better storage efficiency than full snapshots. Each version stores only the delta from the previous version. Full state reconstructed by replaying diffs. |
| Per-file encryption with passphrase-derived key | Allows selective sync and granular access. Compatible with the filesystem-based storage model. User controls encryption per Space. |
| Tables persisted as JSON on disk | SQLite is the runtime store for table data, but a `.json` export lives in the Space filesystem for portability and user access. |

---

## 7. Resolved Questions

| # | Question | Resolution |
|---|----------|------------|
| 1 | Aggregate boundaries for nested folders | Each Folder is its own aggregate. Space is a lightweight root that doesn't load the full tree. |
| 2 | Embedding storage | LanceDB — embedded, columnar, local-first. One instance per Space. |
| 3 | Librarian permission model | Three modes: per-action approval (default), batch approval, and automated standing rules. User chooses. |
| 4 | Version storage strategy | Incremental diffs. Best tradeoff of storage efficiency vs reconstruction cost. |
| 5 | Table persistence format | JSON files on disk alongside SQLite runtime storage. Ensures portability. |
| 6 | Sync protocol | Delta-based sync. Track timestamps, sync deltas, surface conflicts for manual resolution. Simpler than CRDTs/OT and sufficient for the single-user premium sync model. |
| 7 | Encryption implementation | Per-file encryption with passphrase-derived key. Granular, compatible with selective sync. |
| 8 | Cross-Space references | Not supported. Spaces are fully isolated boundaries. |
| 9 | Import/export formats | Deferred to a future phase. |
