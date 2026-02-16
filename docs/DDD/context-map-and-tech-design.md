# Cortex — Context Map & Tactical Design

> **Product:** Cortex
> **Company:** nullbrain
> **Version:** 0.1.0 (Initial Tactical Design)
> **Date:** 2026-02-15
> **Prerequisite:** Domain Definition & Ubiquitous Language v0.2.0

---

## 1. Context Map

### 1.1 Context Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   ┌──────────────┐        ┌──────────────────┐                  │
│   │  Authoring   │──ACL──▶│  Organization    │                  │
│   │   Context    │        │    Context        │                  │
│   └──────┬───────┘        └────────┬─────────┘                  │
│          │                         │                            │
│          │ events                  │ events                     │
│          ▼                         ▼                            │
│   ┌──────────────┐        ┌──────────────────┐                  │
│   │   Storage    │◀──CF──│  Intelligence     │                  │
│   │   Context    │        │    Context        │                  │
│   └──────┬───────┘        └──────────────────┘                  │
│          │                                                      │
│          │ events                                               │
│          ▼                                                      │
│   ┌──────────────┐                                              │
│   │    Sync      │                                              │
│   │   Context    │                                              │
│   └──────────────┘                                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

ACL = Anti-Corruption Layer
CF  = Conformist
```

### 1.2 Relationship Definitions

| Upstream | Downstream | Relationship | Description |
|----------|------------|-------------|-------------|
| **Authoring** | **Storage** | Customer-Supplier | Authoring produces content (documents, tables). Storage persists it. Authoring defines what needs to be stored; Storage fulfills the contract. |
| **Authoring** | **Organization** | ACL | When a document or table is created, Organization needs to know so it can file the item. Organization translates Authoring's `DocumentCreated` event into its own `ItemFiled` command via an anti-corruption layer — it doesn't depend on Authoring's internal model. |
| **Organization** | **Storage** | Customer-Supplier | Organization defines structure (folders, spaces). Storage persists the structural metadata to the Index and mirrors it on the filesystem. |
| **Organization** | **Intelligence** | Published Language | Organization publishes item and tag data that Intelligence consumes for embedding generation and search indexing. Intelligence conforms to Organization's published item/tag schema. |
| **Storage** | **Intelligence** | Conformist | Intelligence reads content from Storage (filesystem + Index) to generate embeddings and answer knowledge queries. Intelligence conforms to Storage's data formats. |
| **Storage** | **Sync** | Customer-Supplier | Sync reads from and writes to Storage. Storage exposes change tracking (timestamps, deltas) that Sync consumes. Sync pushes remote changes back through Storage. |
| **Intelligence** | **Organization** | ACL | The Librarian proposes organizational changes (move, tag, refile). These proposals are translated through an ACL into Organization commands. Organization doesn't know about LLMs. |

### 1.3 Event Flow

```
Document saved in Authoring
  → DocumentUpdated event
  → Storage persists .md file + updates Index
    → FileWritten event
    → Intelligence generates/updates Embedding in LanceDB
  → Organization updates item metadata if needed
    → ItemTagged (if auto-tagging rule active via Librarian)

User moves a folder in Organization
  → FolderMoved event
  → Storage updates filesystem + Index
  → Intelligence re-indexes affected items (paths changed)
  → Sync marks affected items as changed (if in SyncSet)

Sync pulls remote changes
  → Storage applies deltas or surfaces Conflict
  → Intelligence re-embeds changed items
```

---

## 2. Tactical Design

### 2.1 Authoring Context

#### Entities

**Document**
```
Document (Aggregate Root)
├── id: DocumentId (Value Object)
├── title: string
├── content: BlockTree (Value Object — the TipTap document structure)
├── metadata: DocumentMetadata (Value Object)
│   ├── createdAt: DateTime
│   ├── updatedAt: DateTime
│   └── wordCount: number
├── versions: Version[] (managed via VersionHistory)
└── templateId: TemplateId | null
```

**Version**
```
Version (Entity, child of Document)
├── id: VersionId (Value Object)
├── documentId: DocumentId
├── diff: Diff (Value Object — incremental delta from previous version)
├── createdAt: DateTime
└── label: string | null (optional user-provided label, e.g., "Draft 2")
```

**Table**
```
Table (Aggregate Root)
├── id: TableId (Value Object)
├── title: string
├── columns: Column[] (Value Object)
│   ├── id: ColumnId
│   ├── name: string
│   ├── type: ColumnType (text | number | date | select | multiSelect | checkbox | url)
│   └── options: SelectOption[] (for select/multiSelect types)
├── records: Record[] (Entity)
│   ├── id: RecordId
│   └── cells: Map<ColumnId, CellValue>
├── metadata: TableMetadata (Value Object)
│   ├── createdAt: DateTime
│   └── updatedAt: DateTime
└── templateId: TemplateId | null
```

**Template**
```
Template (Aggregate Root)
├── id: TemplateId (Value Object)
├── name: string
├── description: string
├── type: TemplateType (document | table)
├── content: BlockTree | TableSchema (the template blueprint)
├── createdAt: DateTime
└── updatedAt: DateTime
```

#### Value Objects

| Value Object | Properties | Notes |
|-------------|------------|-------|
| `DocumentId` | `value: string (ULID)` | Globally unique, sortable by creation time |
| `TableId` | `value: string (ULID)` | |
| `TemplateId` | `value: string (ULID)` | |
| `VersionId` | `value: string (ULID)` | |
| `RecordId` | `value: string (ULID)` | |
| `ColumnId` | `value: string (ULID)` | |
| `BlockTree` | TipTap JSON document structure | Serialized to/from markdown for persistence |
| `Diff` | `operations: DiffOperation[]` | Represents the incremental change between two versions |
| `CellValue` | `value: string \| number \| boolean \| string[] \| null` | Type depends on column definition |
| `DocumentMetadata` | `createdAt, updatedAt, wordCount` | Computed/tracked automatically |
| `TableMetadata` | `createdAt, updatedAt` | |
| `ColumnType` | Enum: `text, number, date, select, multiSelect, checkbox, url` | Extensible |

#### Domain Services

| Service | Responsibility |
|---------|---------------|
| `VersioningService` | Creates diffs between document states, reconstructs full state from diff chain, manages version pruning |
| `TemplateService` | Instantiates new documents/tables from templates, validates template structure |

#### Repository Interfaces

| Repository | Methods |
|------------|---------|
| `DocumentRepository` | `save(doc)`, `findById(id)`, `findByFolder(folderId)`, `delete(id)` |
| `TableRepository` | `save(table)`, `findById(id)`, `findByFolder(folderId)`, `delete(id)` |
| `TemplateRepository` | `save(template)`, `findAll()`, `findById(id)`, `findByType(type)`, `delete(id)` |
| `VersionRepository` | `save(version)`, `findByDocument(docId)`, `findById(id)`, `getDiffChain(docId, fromVersion?, toVersion?)` |

#### Domain Events

| Event | Payload | Trigger |
|-------|---------|---------|
| `DocumentCreated` | `{ documentId, title, folderId, timestamp }` | New document saved for the first time |
| `DocumentUpdated` | `{ documentId, title, timestamp }` | Document content or metadata changed |
| `DocumentDeleted` | `{ documentId, folderId, timestamp }` | Document removed |
| `DocumentVersionSaved` | `{ documentId, versionId, timestamp, label? }` | Explicit or auto-save version snapshot |
| `TableCreated` | `{ tableId, title, folderId, timestamp }` | New table created |
| `TableUpdated` | `{ tableId, timestamp }` | Table structure or records changed |
| `RecordAdded` | `{ tableId, recordId, timestamp }` | Row added to table |
| `RecordUpdated` | `{ tableId, recordId, timestamp }` | Row modified |
| `RecordDeleted` | `{ tableId, recordId, timestamp }` | Row removed |

---

### 2.2 Organization Context

#### Entities

**Space**
```
Space (Aggregate Root — lightweight)
├── id: SpaceId (Value Object)
├── name: string
├── rootPath: FilePath (Value Object — the filesystem directory)
├── settings: SpaceSettings (Value Object)
│   ├── encryptionEnabled: boolean
│   └── syncEnabled: boolean
├── createdAt: DateTime
└── updatedAt: DateTime
```

Note: Space does NOT contain its folder tree. Folders are loaded independently.

**Folder**
```
Folder (Aggregate Root — independent)
├── id: FolderId (Value Object)
├── spaceId: SpaceId (reference to parent Space)
├── parentFolderId: FolderId | null (null = direct child of Space root)
├── name: string
├── path: FolderPath (Value Object — computed relative path within Space)
├── sortOrder: number
├── createdAt: DateTime
└── updatedAt: DateTime
```

**Item Reference**
```
ItemRef (Entity — the Organization context's view of an item)
├── id: ItemId (Value Object)
├── spaceId: SpaceId
├── folderId: FolderId
├── type: ItemType (document | table | attachment)
├── title: string
├── tags: TagId[]
├── createdAt: DateTime
└── updatedAt: DateTime
```

Note: ItemRef is Organization's projection of items owned by Authoring/Storage. It doesn't contain content — just enough to organize and discover.

**Tag**
```
Tag (Aggregate Root)
├── id: TagId (Value Object)
├── spaceId: SpaceId
├── name: string
├── color: TagColor (Value Object) | null
└── createdAt: DateTime
```

#### Value Objects

| Value Object | Properties | Notes |
|-------------|------------|-------|
| `SpaceId` | `value: string (ULID)` | |
| `FolderId` | `value: string (ULID)` | |
| `ItemId` | `value: string (ULID)` | Shared identity across contexts |
| `TagId` | `value: string (ULID)` | |
| `FilePath` | `value: string` | Absolute path on disk |
| `FolderPath` | `value: string` | Relative path within a Space (e.g., `projects/cortex`) |
| `ItemType` | Enum: `document, table, attachment` | |
| `TagColor` | `value: string (hex)` | |
| `SpaceSettings` | `encryptionEnabled, syncEnabled` | Per-Space configuration |

#### Domain Services

| Service | Responsibility |
|---------|---------------|
| `ItemFilingService` | Handles moving items between folders, validates no cross-Space moves, updates paths |
| `TaggingService` | Manages tag assignment/removal, enforces tag uniqueness within a Space |
| `FolderTreeService` | Queries to build folder tree views (lazy-loaded), computes paths, handles cascading moves |

#### Repository Interfaces

| Repository | Methods |
|------------|---------|
| `SpaceRepository` | `save(space)`, `findById(id)`, `findAll()`, `delete(id)` |
| `FolderRepository` | `save(folder)`, `findById(id)`, `findByParent(parentId)`, `findBySpace(spaceId)`, `delete(id)` |
| `ItemRefRepository` | `save(ref)`, `findById(id)`, `findByFolder(folderId)`, `findByTag(tagId)`, `findBySpace(spaceId)`, `search(spaceId, query)` |
| `TagRepository` | `save(tag)`, `findById(id)`, `findBySpace(spaceId)`, `findByName(spaceId, name)`, `delete(id)` |

#### Domain Events

| Event | Payload | Trigger |
|-------|---------|---------|
| `SpaceCreated` | `{ spaceId, name, rootPath, timestamp }` | New Space initialized |
| `SpaceArchived` | `{ spaceId, timestamp }` | Space archived/deactivated |
| `SpaceSettingsUpdated` | `{ spaceId, settings, timestamp }` | Encryption or sync settings changed |
| `FolderCreated` | `{ folderId, spaceId, parentFolderId, name, timestamp }` | New folder created |
| `FolderMoved` | `{ folderId, oldParentId, newParentId, timestamp }` | Folder relocated |
| `FolderDeleted` | `{ folderId, spaceId, timestamp }` | Folder removed |
| `ItemFiled` | `{ itemId, spaceId, folderId, itemType, timestamp }` | New item placed in a folder |
| `ItemMoved` | `{ itemId, oldFolderId, newFolderId, timestamp }` | Item relocated |
| `ItemTagged` | `{ itemId, tagId, timestamp }` | Tag applied to item |
| `ItemUntagged` | `{ itemId, tagId, timestamp }` | Tag removed from item |

---

### 2.3 Storage Context

#### Entities

**IndexEntry**
```
IndexEntry (Aggregate Root)
├── id: ItemId (Value Object — shared identity with other contexts)
├── spaceId: SpaceId
├── relativePath: string (path within the Space directory)
├── fileType: FileType (Value Object)
├── sizeBytes: number
├── checksum: string (content hash for change detection)
├── encryptedFlag: boolean
├── createdAt: DateTime
├── updatedAt: DateTime
└── syncState: SyncState (Value Object)
    ├── lastSyncedAt: DateTime | null
    ├── dirty: boolean
    └── version: number (monotonic counter for delta tracking)
```

#### Value Objects

| Value Object | Properties | Notes |
|-------------|------------|-------|
| `FileType` | Enum: `markdown, json, image, pdf, csv, binary` | Determines how Storage handles the file |
| `SyncState` | `lastSyncedAt, dirty, version` | Tracks whether this entry has unsynced changes |
| `Checksum` | `value: string (SHA-256)` | Used for change detection and conflict identification |

#### Domain Services

| Service | Responsibility |
|---------|---------------|
| `FileSystemService` | Reads/writes files to the Space directory, ensures directory structure mirrors Organization hierarchy, handles file moves/renames |
| `IndexService` | CRUD operations on the SQLite index, maintains consistency between index and filesystem |
| `EncryptionService` | Encrypts/decrypts individual files using passphrase-derived key (AES-256-GCM). Handles key derivation (Argon2id). |
| `ChangeTrackingService` | Computes checksums, marks entries as dirty, increments version counters for sync |
| `TableExportService` | Exports Table data from SQLite to `.json` files in the Space filesystem. Runs on table save. |

#### Repository Interfaces

| Repository | Methods |
|------------|---------|
| `IndexEntryRepository` | `save(entry)`, `findById(id)`, `findBySpace(spaceId)`, `findDirty(spaceId)`, `findByPath(spaceId, path)`, `delete(id)` |

#### Domain Events

| Event | Payload | Trigger |
|-------|---------|---------|
| `FileWritten` | `{ itemId, spaceId, relativePath, fileType, checksum, timestamp }` | File created or updated on disk |
| `FileDeleted` | `{ itemId, spaceId, relativePath, timestamp }` | File removed from disk |
| `FileMoved` | `{ itemId, oldPath, newPath, timestamp }` | File relocated on disk |
| `AttachmentImported` | `{ itemId, spaceId, relativePath, fileType, sizeBytes, timestamp }` | External file brought into a Space |
| `IndexUpdated` | `{ itemId, spaceId, timestamp }` | Index entry modified |

---

### 2.4 Intelligence Context

#### Entities

**Provider**
```
Provider (Aggregate Root)
├── id: ProviderId (Value Object)
├── name: string
├── type: ProviderType (local | cloudApi | nullbrainHosted)
├── config: ProviderConfig (Value Object)
│   ├── endpoint: string (URL or local socket)
│   ├── modelId: string
│   ├── apiKey: string | null (encrypted, for cloud APIs)
│   └── maxTokens: number
├── capabilities: ProviderCapability[] (chat | embedding)
├── isDefault: boolean
└── createdAt: DateTime
```

**KnowledgeChat**
```
KnowledgeChat (Aggregate Root)
├── id: ChatId (Value Object)
├── spaceId: SpaceId
├── providerId: ProviderId
├── title: string
├── messages: ChatMessage[] (Entity)
│   ├── id: MessageId
│   ├── role: MessageRole (user | assistant)
│   ├── content: string
│   ├── sources: SourceReference[] (Value Object — items retrieved for context)
│   └── timestamp: DateTime
├── createdAt: DateTime
└── updatedAt: DateTime
```

**Librarian**
```
Librarian (Aggregate Root — singleton per Space)
├── spaceId: SpaceId
├── providerId: ProviderId
├── mode: LibrarianMode (manual | batch | automated)
├── rules: LibrarianRule[] (Entity)
│   ├── id: RuleId
│   ├── trigger: RuleTrigger (onItemCreated | onSchedule | onDemand)
│   ├── action: RuleAction (autoTag | suggestMove | suggestLink)
│   ├── config: Record<string, any> (rule-specific parameters)
│   └── enabled: boolean
├── proposals: LibrarianProposal[] (Entity)
│   ├── id: ProposalId
│   ├── type: ProposalType (tag | move | link)
│   ├── description: string
│   ├── targetItemId: ItemId
│   ├── suggestedAction: Record<string, any>
│   ├── status: ProposalStatus (pending | approved | rejected | executed)
│   └── createdAt: DateTime
└── lastRunAt: DateTime | null
```

#### Value Objects

| Value Object | Properties | Notes |
|-------------|------------|-------|
| `ProviderId` | `value: string (ULID)` | |
| `ChatId` | `value: string (ULID)` | |
| `MessageId` | `value: string (ULID)` | |
| `RuleId` | `value: string (ULID)` | |
| `ProposalId` | `value: string (ULID)` | |
| `ProviderType` | Enum: `local, cloudApi, nullbrainHosted` | |
| `ProviderCapability` | Enum: `chat, embedding` | A provider may support one or both |
| `MessageRole` | Enum: `user, assistant` | |
| `LibrarianMode` | Enum: `manual, batch, automated` | Default: manual |
| `RuleTrigger` | Enum: `onItemCreated, onSchedule, onDemand` | When the rule fires |
| `RuleAction` | Enum: `autoTag, suggestMove, suggestLink` | What the rule does |
| `ProposalStatus` | Enum: `pending, approved, rejected, executed` | |
| `SourceReference` | `{ itemId, title, relevanceScore, snippet }` | References to items used as RAG context |
| `ProviderConfig` | `endpoint, modelId, apiKey, maxTokens` | Connection details for an LLM backend |

#### Domain Services

| Service | Responsibility |
|---------|---------------|
| `EmbeddingService` | Generates embeddings for items using the configured embedding Provider, stores them in LanceDB |
| `KnowledgeQueryService` | Combines SQLite FTS and LanceDB vector search to retrieve relevant items for a query |
| `ChatOrchestrator` | Manages the RAG pipeline: receives user message → retrieves context via KnowledgeQueryService → sends to Provider → stores response |
| `LibrarianOrchestrator` | Runs the librarian analysis: scans items → generates proposals via Provider → queues for approval or auto-executes based on rules |

#### Repository Interfaces

| Repository | Methods |
|------------|---------|
| `ProviderRepository` | `save(provider)`, `findById(id)`, `findAll()`, `findDefault()`, `findByCapability(cap)`, `delete(id)` |
| `KnowledgeChatRepository` | `save(chat)`, `findById(id)`, `findBySpace(spaceId)`, `delete(id)` |
| `LibrarianRepository` | `save(librarian)`, `findBySpace(spaceId)` |
| `EmbeddingRepository` | `upsert(itemId, vector)`, `delete(itemId)`, `search(queryVector, topK)`, `findByItem(itemId)` |

#### Domain Events

| Event | Payload | Trigger |
|-------|---------|---------|
| `EmbeddingGenerated` | `{ itemId, spaceId, providerId, timestamp }` | Embedding created or updated for an item |
| `KnowledgeQueried` | `{ spaceId, query, resultCount, timestamp }` | A search was executed |
| `ChatMessageSent` | `{ chatId, messageId, role, timestamp }` | User sent a message in Knowledge Chat |
| `ChatResponseGenerated` | `{ chatId, messageId, providerId, sourceCount, timestamp }` | LLM responded with retrieved context |
| `LibrarianProposalCreated` | `{ proposalId, spaceId, type, targetItemId, timestamp }` | Librarian generated an organizational suggestion |
| `LibrarianActionApproved` | `{ proposalId, timestamp }` | User approved a proposal |
| `LibrarianActionRejected` | `{ proposalId, timestamp }` | User rejected a proposal |
| `LibrarianActionExecuted` | `{ proposalId, timestamp }` | Approved or auto-approved action was carried out |
| `LibrarianRuleCreated` | `{ ruleId, spaceId, trigger, action, timestamp }` | User created a standing automation rule |

---

### 2.5 Sync Context

#### Entities

**SyncConnection**
```
SyncConnection (Aggregate Root)
├── id: SyncConnectionId (Value Object)
├── spaceId: SpaceId
├── serviceUrl: string (nullbrain Sync Service endpoint)
├── authToken: string (encrypted)
├── status: ConnectionStatus (connected | disconnected | error)
├── lastSyncAt: DateTime | null
└── createdAt: DateTime
```

**SyncSet**
```
SyncSet (Aggregate Root)
├── id: SyncSetId (Value Object)
├── spaceId: SpaceId
├── connectionId: SyncConnectionId
├── includedPaths: FolderPath[] (folders selected for sync)
├── excludedPatterns: string[] (glob patterns to exclude, e.g., "*.tmp")
└── updatedAt: DateTime
```

**Conflict**
```
Conflict (Aggregate Root)
├── id: ConflictId (Value Object)
├── spaceId: SpaceId
├── itemId: ItemId
├── localVersion: ConflictSnapshot (Value Object)
│   ├── content: string | BinaryRef
│   ├── checksum: string
│   └── modifiedAt: DateTime
├── remoteVersion: ConflictSnapshot (Value Object)
│   ├── content: string | BinaryRef
│   ├── checksum: string
│   └── modifiedAt: DateTime
├── status: ConflictStatus (pending | resolvedLocal | resolvedRemote | resolvedMerge)
├── resolvedAt: DateTime | null
└── detectedAt: DateTime
```

#### Value Objects

| Value Object | Properties | Notes |
|-------------|------------|-------|
| `SyncConnectionId` | `value: string (ULID)` | |
| `SyncSetId` | `value: string (ULID)` | |
| `ConflictId` | `value: string (ULID)` | |
| `ConnectionStatus` | Enum: `connected, disconnected, error` | |
| `ConflictStatus` | Enum: `pending, resolvedLocal, resolvedRemote, resolvedMerge` | |
| `ConflictSnapshot` | `content, checksum, modifiedAt` | Captures one side of a conflict |
| `SyncDelta` | `{ itemId, version, operation (create\|update\|delete), checksum, timestamp }` | A single change to be synced |

#### Domain Services

| Service | Responsibility |
|---------|---------------|
| `SyncOrchestrator` | Coordinates the sync cycle: collect local deltas → push to service → pull remote deltas → apply or surface conflicts |
| `ConflictResolver` | Presents conflicts to the user, applies their resolution choice (keep local, keep remote, manual merge) |
| `DeltaCollector` | Queries Storage's change tracking to build the set of SyncDeltas since last sync |
| `SyncAuthService` | Manages authentication with the nullbrain Sync Service (token refresh, credential storage) |

#### Repository Interfaces

| Repository | Methods |
|------------|---------|
| `SyncConnectionRepository` | `save(conn)`, `findBySpace(spaceId)`, `delete(id)` |
| `SyncSetRepository` | `save(set)`, `findBySpace(spaceId)` |
| `ConflictRepository` | `save(conflict)`, `findPending(spaceId)`, `findById(id)`, `resolve(id, resolution)` |

#### Domain Events

| Event | Payload | Trigger |
|-------|---------|---------|
| `SyncStarted` | `{ spaceId, connectionId, timestamp }` | Sync cycle initiated |
| `SyncCompleted` | `{ spaceId, pushedCount, pulledCount, conflictCount, timestamp }` | Sync cycle finished |
| `ConflictDetected` | `{ conflictId, spaceId, itemId, timestamp }` | Divergent versions found during sync |
| `ConflictResolved` | `{ conflictId, resolution, timestamp }` | User resolved a conflict |

---

## 3. Cross-Context Integration Patterns

### 3.1 Shared Kernel: ItemId

All contexts share the `ItemId` value object. This is the one piece of shared identity that allows contexts to reference the same logical item without coupling their internal models. Generated as a ULID at creation time in the Authoring context and propagated via events.

### 3.2 Event Bus (In-Process)

Since Cortex is a local-first desktop/mobile app, the event bus is in-process (not a distributed message broker). Events are dispatched synchronously or via a lightweight async queue within the application.

| Pattern | Usage |
|---------|-------|
| **Synchronous dispatch** | When downstream must complete before the user sees a result (e.g., Storage persisting after Authoring saves) |
| **Async queue** | When downstream work is best-effort or can lag (e.g., Intelligence re-embedding after a save, Sync marking items dirty) |

### 3.3 Anti-Corruption Layers

| ACL | Between | Purpose |
|-----|---------|---------|
| **Authoring → Organization** | Translates `DocumentCreated` / `TableCreated` into `ItemFiled` commands. Organization doesn't know about Documents or Tables — only ItemRefs. |
| **Intelligence → Organization** | Translates `LibrarianActionExecuted` into Organization commands (`ItemMoved`, `ItemTagged`). Organization doesn't know about the Librarian. |

### 3.4 Event Subscription Matrix

| Event | Authoring | Organization | Storage | Intelligence | Sync |
|-------|-----------|-------------|---------|-------------|------|
| `DocumentCreated` | — | ✓ (file item) | ✓ (persist) | ✓ (embed) | — |
| `DocumentUpdated` | — | ✓ (update ref) | ✓ (persist) | ✓ (re-embed) | — |
| `DocumentDeleted` | — | ✓ (remove ref) | ✓ (delete file) | ✓ (remove embedding) | — |
| `TableCreated` | — | ✓ (file item) | ✓ (persist + export JSON) | ✓ (embed) | — |
| `TableUpdated` | — | ✓ (update ref) | ✓ (persist + export JSON) | ✓ (re-embed) | — |
| `FolderMoved` | — | — | ✓ (move dir) | ✓ (re-index paths) | ✓ (mark dirty) |
| `FolderDeleted` | — | — | ✓ (delete dir) | ✓ (remove embeddings) | ✓ (mark dirty) |
| `ItemMoved` | — | — | ✓ (move file) | ✓ (update index) | ✓ (mark dirty) |
| `ItemTagged` | — | — | — | ✓ (update metadata) | ✓ (mark dirty) |
| `FileWritten` | — | — | — | — | ✓ (mark dirty) |
| `AttachmentImported` | — | ✓ (file item) | — | ✓ (embed) | — |
| `LibrarianActionExecuted` | — | ✓ (apply action) | — | — | — |
| `ConflictResolved` | — | — | ✓ (apply resolution) | ✓ (re-embed) | — |

---

## 4. File System Layout

```
~/cortex/
├── spaces/
│   ├── personal/                          ← Space root directory
│   │   ├── .cortex/                       ← Hidden metadata directory
│   │   │   ├── index.db                   ← SQLite database (Index)
│   │   │   ├── lance/                     ← LanceDB directory (Embedding Store)
│   │   │   ├── sync.json                  ← Sync configuration (SyncSet, connection info)
│   │   │   └── librarian.json             ← Librarian rules and state
│   │   ├── projects/                      ← Folder
│   │   │   ├── cortex/                    ← Nested Folder
│   │   │   │   ├── architecture.md        ← Document
│   │   │   │   ├── reading-list.json      ← Table (JSON export)
│   │   │   │   └── diagram.png            ← Attachment
│   │   │   └── nullbrain/
│   │   │       └── ...
│   │   ├── journal/
│   │   │   ├── 2026-02-15.md
│   │   │   └── ...
│   │   └── reference/
│   │       └── ...
│   └── work/                              ← Another Space
│       ├── .cortex/
│       │   ├── index.db
│       │   ├── lance/
│       │   └── ...
│       └── ...
├── config/
│   ├── providers.json                     ← LLM Provider configurations (global)
│   ├── templates/                         ← User templates (global)
│   └── settings.json                      ← App-level settings
└── logs/
    └── cortex.log
```
