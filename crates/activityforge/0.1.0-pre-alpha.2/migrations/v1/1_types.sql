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
  'Public',
  'Visit',
  'Report',
  'Triage',
  'Write',
  'Maintain',
  'Admin',
  'Delegate',
  'Deny'
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
  'accept_activity',
  'create_activity',
  'follow_activity',
  'like_activity',
  'object',
  'repository',
  'application',
  'person',
  'oauth_grant',
  'oauth_token',
  'oauth_client'
);

create type filter_key as enum (
  'members',
  'parent',
  'subteams',
  'oversees',
  'overseen_by',
  'project'
);

create type collab_relationship as enum (
  'hasCollaborator',
  'hasMember'
);

create type role_filter as (
  key   filter_key,
  value role
);

create type table_entry as (
  entry_type table_type,
  id         uuid
);

create type challenge_method as enum (
  'plain',
  'S256'
);

create type code_challenge as (
  code   bytea,
  method challenge_method
);

create type scope as enum (
  'profile',
  'push',
  'visit',
  'report',
  'triage',
  'maintain',
  'delegate',
  'register',
  'read',
  'read:accounts',
  'read:blocks',
  'read:bookmarks',
  'read:favourites',
  'read:filters',
  'read:follows',
  'read:lists',
  'read:mutes',
  'read:notifications',
  'read:search',
  'read:statuses',
  'write',
  'write:accounts',
  'write:blocks',
  'write:bookmarks',
  'write:favourites',
  'write:filters',
  'write:follows',
  'write:lists',
  'write:mutes',
  'write:notifications',
  'write:search',
  'write:statuses',
  'admin',
  'admin:read',
  'admin:read:accounts',
  'admin:read:canonical_email_blocks',
  'admin:read:domain_allows',
  'admin:read:domain_blocks',
  'admin:read:email_domain_blocks',
  'admin:read:ip_blocks',
  'admin:read:reports',
  'admin:write',
  'admin:write:accounts',
  'admin:write:canonical_email_blocks',
  'admin:write:domain_allows',
  'admin:write:domain_blocks',
  'admin:write:email_domain_blocks',
  'admin:write:ip_blocks',
  'admin:write:reports'
);

create type oauth_token_type as enum (
  'access',
  'refresh',
  'register'
);

create type oauth_client_id as enum (
  'activityforge',
  'anvil',
  'forgejo'
);

create type oauth_client_type as enum (
  'public',
  'private'
);

create type oauth_grant_type as enum (
  'authorization_code',
  'refresh_token'
);
