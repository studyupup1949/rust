create table if not exists inbox
(
  uuid       uuid           primary key default gen_random_uuid(),
  id         text           unique not null,
  actor      table_entry    unique not null,
  activities table_entry[]
);

create table if not exists outbox
(
  uuid       uuid           primary key default gen_random_uuid(),
  id         text           unique not null,
  actor      table_entry    unique not null,
  activities table_entry[]
);

create table if not exists follower
(
  uuid      uuid          primary key default gen_random_uuid(),
  id        text          unique not null,
  actor     table_entry   unique not null,
  following table_entry[]
);

create table if not exists key
(
  uuid       uuid        primary key default gen_random_uuid(),
  id         text        unique not null,
  actor_id   text        not null,
  key        bytea       unique not null,
  key_type   key_type    not null,
  is_private bool        not null default false,
  actor      table_entry not null
);

create table if not exists object
(
  uuid    uuid        primary key default gen_random_uuid(),
  kind    object_type not null,
  id      text        unique not null,
  name    text,
  content text,
  summary text
);

create table if not exists activity
(
  uuid       uuid          primary key default gen_random_uuid(),
  kind       activity_type not null,
  id         text          unique not null,
  name       text,
  content    text,
  summary    text,
  actor      table_entry   not null,
  object     uuid          references object(uuid) on delete set null,
  origin     uuid          references object(uuid) on delete set null,
  target     uuid          references object(uuid) on delete set null,
  instrument uuid          references object(uuid) on delete set null,
  result     uuid          references object(uuid) on delete set null
);

create table if not exists like_activity (
  uuid   uuid primary key default gen_random_uuid(),
  /* Represents the IRI used to fetch the Like */
  id     text unique not null,
  /* References the actor record of the actor who published the `Like` */
  actor  text not null,
  /* References the resource being liked */
  object text not null
);

create table if not exists accept_activity (
  uuid   uuid         primary key default gen_random_uuid(),
  /* Represents the IRI used to fetch the activity */
  id     text         unique not null,
  /* References the actor record of the actor who published the activity */
  actor  table_entry  not null,
  /* References the resource being accepted */
  object table_entry  not null
);

create table if not exists create_activity (
  uuid   uuid         primary key default gen_random_uuid(),
  /* Represents the IRI used to fetch the activity */
  id     text         unique not null,
  /* References the actor record of the actor who published the activity */
  actor  table_entry  not null,
  /* References the resource being created */
  object table_entry  not null
);

create table if not exists follow_activity (
  uuid   uuid         primary key default gen_random_uuid(),
  /* Represents the IRI used to fetch the activity */
  id     text         unique not null,
  /* References the actor record of the actor who published the activity */
  actor  table_entry  not null,
  /* References the resource being followed */
  object table_entry  not null
);

create table if not exists team
(
  uuid        uuid        primary key default gen_random_uuid(),
  id          text        unique not null,
  name        text        unique not null,
  content     text,
  summary     text,
  /* References the parent team(uuid) of this team */
  context     uuid        references team(uuid) on delete set null,
  /* References the activity inbox */
  inbox       uuid        unique not null references inbox(uuid) on delete cascade,
  /* References the activity outbox */
  outbox      uuid        unique not null references outbox(uuid) on delete cascade,
  /* time */
  published   timestamptz not null default now(),
  /* References collaborator(uuid) members of this team */
  members     uuid[],
  /* References team(uuid) subteams of this team */
  subteams    uuid[],
  /* References to team(uuid) overseen by this team */
  oversees    uuid[],
  /* References to team(uuid) overseeing this team */
  overseen_by uuid[],
  /* References to role_filter(uuid) for role maps an actor(uuid) to a role */
  role_filter role_filter[],
  /* References to key(uuid) records for this team */
  key_ids     uuid[]
);

create table if not exists role_grant
(
  uuid         uuid         primary key default gen_random_uuid(),
  /* Represents the IRI used to fetch the Grant */
  id           text         unique not null,
  /* References the actor record of the resource that is granted access */
  actor        table_entry  not null,
  /* Represents the role used to fine-tune access to the filtered resource */
  objects      role[]       not null,
  /* References the resource being given access by the grant */
  context      table_entry  not null,
  /* References the actor record that inherits the role */
  target_entry table_entry  unique,
  /* References the activity that triggered the Grant */
  fulfills     table_entry  unique,
  start_time   timestamptz,
  end_time     timestamptz
);

create table if not exists factory
(
  uuid                  uuid          primary key default gen_random_uuid(),
  id                    text          unique not null,
  name                  text          unique not null,
  available_actor_types actor_type[],
  inbox                 uuid          unique not null references inbox(uuid) on delete cascade,
  outbox                uuid          unique not null references outbox(uuid) on delete cascade,
  /* IRI to download the collection of collaborators */
  collaborators_id      text,
  /* References to collaborator(uuid) records for this factory */
  collaborators         uuid[],
  /* IRI to download the collection of followers */
  followers_id          text,
  /* References to follower(uuid) records for this factory */
  followers             uuid[],
  /* IRI to download the collection of teams */
  teams_id              text,
  /* References to team(uuid) records for this factory */
  teams                 uuid[],
  /* References to key(uuid) records for this factory */
  key_ids               uuid[]
);

create table if not exists patch_tracker
(
  uuid      uuid    primary key default gen_random_uuid(),
  id        text    unique not null,
  name      text    unique not null,
  summary   text,
  content   text,
  inbox     uuid    unique not null references inbox(uuid) on delete cascade,
  outbox    uuid    unique not null references outbox(uuid) on delete cascade,
  /* References to follower(uuid) records for this patch tracker */
  followers uuid[],
  /* References to key(uuid) records for this patch tracker */
  key_ids   uuid[]
);

create table if not exists ticket_tracker
(
  uuid      uuid    primary key default gen_random_uuid(),
  id        text    unique not null,
  name      text    unique not null,
  summary   text,
  content   text,
  inbox     uuid    unique not null references inbox(uuid) on delete cascade,
  outbox    uuid    unique not null references outbox(uuid) on delete cascade,
  /* References to follower(uuid) records for this ticket tracker */
  followers uuid[],
  /* References to key(uuid) records for this ticket tracker */
  key_ids   uuid[]
);

create table if not exists repository
(
  uuid               uuid    primary key default gen_random_uuid(),
  id                 text    unique not null,
  name               text    unique not null,
  inbox              uuid    unique not null references inbox(uuid) on delete cascade,
  outbox             uuid    unique not null references outbox(uuid) on delete cascade,
  /* List of URIs for cloning the repo */
  clone_uris         text[],
  /* List of URIs for pushing to the repo */
  push_uris          text[],
  forks_id           text    unique,
  /* References to repository(uuid) forks of this repo */
  forks              text[],
  likes_id           text    unique,
  /* References to like(uuid) records for this repo */
  likes              uuid[],
  /* References the IRI used to fetch the list of follower records */
  followers_id       text    unique,
  /* References to follower(uuid) records for this repo */
  followers          text[],
  /* References to key(uuid) records for this repo */
  key_ids            uuid[],
  send_patches_to    text    references patch_tracker(id) on delete set null,
  tickets_tracked_by text    references ticket_tracker(id) on delete set null,
  is_archived        bool,
  moved_to           text    references repository(id) on delete set null,
  mirrors            text    references repository(id) on delete set null,
  team               text    references team(id) on delete set null,
  is_private         boolean not null default false
);

create table if not exists account
(
  uuid         uuid    primary key default gen_random_uuid(),
  id           text    unique not null,
  name         text    unique not null,
  password     text    unique not null,
  /* References the account inbox */
  inbox        uuid    unique not null references inbox(uuid) on delete cascade,
  /* References the account outbox */
  outbox       uuid    unique not null references outbox(uuid) on delete cascade,
  /* Represents the full list of OAuth 2.0 scopes granted to the account */
  scopes       scope[],
  /* References to key(uuid) records for this account */
  key_ids      uuid[]
);

create table if not exists person
(
  uuid         uuid    primary key default gen_random_uuid(),
  id           text    unique not null,
  name         text    unique not null,
  password     text,
  scopes       scope[],
  content      text,
  summary      text,
  /* References the person inbox */
  inbox        uuid    unique not null references inbox(uuid) on delete cascade,
  /* References the person outbox */
  outbox       uuid    unique not null references outbox(uuid) on delete cascade,
  /* IRI that references the list of followers */
  followers_id text    unique,
  /* References to follower(uuid) records for this person */
  followers    uuid[],
  /* References to key(uuid) records for this person */
  key_ids      uuid[],
  is_private   boolean
);

create table if not exists collaborator
(
  uuid         uuid                 primary key default gen_random_uuid(),
  id           text                 unique not null,
  subject      text                 not null,
  relationship collab_relationship,
  object       text                 not null,
  tag          role
);

create table if not exists application
(
  uuid         uuid    primary key default gen_random_uuid(),
  id           text    unique not null,
  name         text    unique not null,
  password     text,
  scopes       scope[],
  content      text,
  summary      text,
  /* References the activity inbox */
  inbox        uuid    unique not null references inbox(uuid) on delete cascade,
  /* References the activity outbox */
  outbox       uuid    unique not null references outbox(uuid) on delete cascade,
  /* IRI that references the list of followers */
  followers_id text    unique,
  /* References to follower(uuid) records for this person */
  followers    uuid[],
  /* References to key(uuid) records for this person */
  key_ids      uuid[]
);

create table if not exists oauth_grant
(
  uuid           uuid            primary key default gen_random_uuid(),
  owner_id       text            unique not null,
  client_id      uuid            unique not null,
  scopes         scope[]         not null,
  redirect_uri   text,
  until          timestamptz,
  pkce           text,
  tag            text            unique not null
);

create table if not exists oauth_token
(
  uuid          uuid             primary key default gen_random_uuid(),
  token         text             unique not null,
  refresh_token text             unique,
  until         timestamptz,
  expires_in    int8,
  scope         scope[],
  token_type    oauth_token_type not null,
  grant_id      uuid             references oauth_grant(uuid) on delete cascade
);

create table if not exists oauth_client
(
  uuid          uuid                primary key default gen_random_uuid(),
  owner_id      uuid                not null references person(uuid) on delete cascade,
  client_type   oauth_client_type   not null,
  scopes        scope[],
  password      text                unique not null,
  issued_at     timestamptz         not null,
  redirect_uris text[],
  grant_types   oauth_grant_type[],
  key_ids       uuid[]
);
