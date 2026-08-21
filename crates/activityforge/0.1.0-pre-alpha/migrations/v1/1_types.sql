create type activity_type as enum (
    'Edit',
    'Grant',
    'RoleFilter',
    'Revoke',
    'Push',
    'Assign',
    'Resolve',
    'Apply'
    'Accept',
    'Add',
    'Announce',
    'Arrive',
    'Block',
    'Create',
    'Delete',
    'Dislike',
    'Flag',
    'Follow',
    'Ignore',
    'Invite',
    'Join',
    'Leave',
    'Like',
    'Listen',
    'Move',
    'Offer',
    'Question',
    'Reject',
    'Read',
    'Remove',
    'TentativeReject',
    'TentativeAccept',
    'Travel',
    'Undo',
    'Update',
    'View'
);

create type actor_type as enum (
    'Factory',
    'Repository',
    'PatchTracker',
    'ReleaseTracker',
    'Roadmap',
    'TicketTracker',
    'Project',
    'Team',
    'Workflow',
    'Follower',
    'Application',
    'Group',
    'Organization',
    'Person',
    'Service'
);

create type object_type as enum (
    'CapabilityUsage',
    'Role',
    'Branch',
    'Commit',
    'Patch',
    'TicketDependency',
    'Ticket',
    'Enum',
    'EnumValue',
    'Field',
    'FieldType',
    'FieldValue',
    'Milestone',
    'Release',
    'ReviewVerdict',
    'ReviewStatus',
    'ReviewThread',
    'Suggestion',
    'CodeQuote',
    'Approval',
    'DiffSide',
    'Review',
    'SshPublicKey',
    'Article',
    'Audio',
    'Document',
    'Event',
    'Image',
    'Note',
    'Page',
    'Place',
    'Profile',
    'Relationship',
    'Tombstone',
    'Video'
);

create type key_type as enum (
    'Ecdsa256',
    'Ecdsa384',
    'Ed25519',
    'Bls12',
    'Sm2',
    'Rsa2048',
    'Rsa3072',
    'Rsa4096'
);

create type role as enum (
    'Visit',
    'Report',
    'Triage',
    'Write',
    'Maintain',
    'Admin',
    'Delegate'
);

create type table_type as enum (
    'inbox',
    'outbox',
    'collaborator',
    'follower',
    'key',
    'role_grant',
    'team',
    'factory',
    'patch_tracker',
    'ticket_tracker',
    'activity',
    'object',
    'repository',
    'person'
);

create type table_entry as (
  entry_type table_type,
  id         uuid
);
