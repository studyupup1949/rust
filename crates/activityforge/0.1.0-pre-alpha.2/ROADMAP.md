# Roadmap

Below is a basic overview of the project development roadmap.

## Data Structures

All `ForgeFed` networking data structures (`JSON-LD` data types used for communicating over `ActivityPub`) are currently implemented.

A number of data structures for persistent storage have been implemented, along with facilities for defining new structures.

However, a large number of persistent storage data structures remain to be implemented.

The following checklist is a break-down of what has been implemented, and what is left to be done.

### Actors

- [x] `Factory`
  - [x] JSON-LD
  - [x] Database
- [x] `Repository`
  - [x] JSON-LD
  - [x] Database
- [x] `Person`
  - [x] JSON-LD
  - [x] Database
- [x] `Team`
  - [x] JSON-LD
  - [x] Database
- [ ] `PatchTracker`
  - [x] JSON-LD
  - [ ] Database
- [ ] `TicketTracker`
  - [x] JSON-LD
  - [ ] Database
- [ ] `ReleaseTracker`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Project`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Roadmap`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Workflow`
  - [x] JSON-LD
  - [ ] Database

### Activities

- [x] `Grant`
  - [x] JSON-LD
  - [x] Database
- [x] `Like`
  - [x] JSON-LD
  - [x] Database
- [ ] `Apply`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Assign`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Edit`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Push`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Resolve`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Revoke`
  - [x] JSON-LD
  - [ ] Database

### Objects

- [x] `Collaborator`
  - [x] JSON-LD
  - [x] Database
- [x] `Follower`
  - [x] JSON-LD
  - [x] Database
- [x] `Key`
  - [x] JSON-LD
  - [x] Database
- [x] `Inbox`
  - [x] JSON-LD
  - [x] Database
- [x] `Outbox`
  - [x] JSON-LD
  - [x] Database
- [x] `Enum`
  - [x] JSON-LD
- [x] `EnumValue`
  - [x] JSON-LD
- [x] `Field`
  - [x] JSON-LD
- [x] `FieldType`
  - [x] JSON-LD
- [x] `FieldValue`
  - [x] JSON-LD
- [x] `FieldValueItem`
  - [x] JSON-LD
- [x] `Role`
  - [x] JSON-LD
  - [x] Database
- [ ] `Branch`
  - [x] JSON-LD
  - [ ] Database
- [ ] `CapabilityUsage`
  - [x] JSON-LD
  - [ ] Database
- [ ] `CodeQuote`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Comment`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Commit`
  - [x] JSON-LD
  - [ ] Database
- [ ] `DependencyRelationship`
  - [x] JSON-LD
  - [ ] Database
- [ ] `DiffSide`
  - [x] JSON-LD
  - [ ] Database
- [ ] `DiffSide`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Milestone`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Patch`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Release`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Review`
  - [x] JSON-LD
  - [ ] Database
- [ ] `SshPublicKey`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Suggestion`
  - [x] JSON-LD
  - [ ] Database
- [ ] `TicketDependency`
  - [x] JSON-LD
  - [ ] Database
- [ ] `Ticket`
  - [x] JSON-LD
  - [ ] Database

## Storage

- [x] Database creation
- [x] Database integration test-suite
  - `podman`-based local integration test-suite
  - contains a number of tests for implemented database types
  - as new types are implemented, corresponding tests will be added
- [x] Encryption of server private key records
  - the server stores some private keys used to sign `ActivityPub` requests
  - private key data should be encrypted in the database, independent of user configuration
  - for example, not relying on database-specific encryption features, full-disk encryption, etc.
- [ ] CI configuration for integration test-suite
  - the goal is to use service-based CI images to avoid spinning up Docker-in-Docker CI images

## Networking

- [ ] `Factory` management of resources, initiated by S2S and/or C2S requests
  - [ ] `Grant`-based access control to server resources
  - [ ] `Repository` management + federation
  - [ ] `Issue` management + federation
  - [ ] `Patch` management + federation
  - [ ] `User` management + federation
  - [ ] `Team` management + federation
  - [ ] `Project` management + federation
  - [ ] `Release` management + federation
  - [ ] `Workflow` management + federation

## UI/UX

- [ ] Login page
- [ ] Account creation
  - [ ] account control panel
- [ ] Admin control panel
- [ ] Moderation settings
- [ ] `Repository` management
- [ ] `Issue` management
- [ ] `Patch` management
- [ ] `Release` management
- [ ] `Workflow` management
- [ ] Federation settings management
