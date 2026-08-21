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

create table if not exists collaborator
(
    uuid         uuid        primary key default gen_random_uuid(),
    subject      table_entry unique not null,
    object       table_entry unique not null,
    instrument   role        not null,
    tag          text
);

create table if not exists follower
(
    uuid           uuid           primary key default gen_random_uuid(),
    actor          table_entry    unique not null,
    following      table_entry[]
);

create table if not exists key
(
    uuid       uuid        primary key default gen_random_uuid(),
    id         text        unique not null,
    key        bytea       unique not null,
    key_type   key_type    not null,
    is_private bool        not null default false,
    actor      table_entry not null
);

create table if not exists object
(
    uuid        uuid        primary key default gen_random_uuid(),
    kind        object_type not null,
    id          text        unique not null,
    name        text,
    content     text,
    summary     text
);

create table if not exists activity
(
    uuid       uuid          primary key default gen_random_uuid(),
    kind       activity_type not null,
    id         text          unique not null,
    name       text,
    content    text,
    summary    text,
    object     uuid          references object(uuid),
    origin     uuid          references object(uuid),
    target     uuid          references object(uuid),
    instrument uuid          references object(uuid),
    result     uuid          references object(uuid)
);

create table if not exists like_activity (
    uuid   uuid         primary key default gen_random_uuid(),
    /* Represents the IRI used to fetch the Like */
    id     text         unique not null,
    /* References the actor record of the actor who published the `Like` */
    actor  table_entry  unique not null,
    /* References the resource being liked */
    object table_entry  not null
);

create table if not exists role_grant
(
    uuid        uuid         primary key default gen_random_uuid(),
    /* Represents the IRI used to fetch the Grant */
    id          text         unique not null,
    /* References the actor record of the resource that is granted access */
    actor       table_entry  unique not null,
    /* Represents the role used to fine-tune access to the filtered resource */
    object      role         not null,
    /* References the resource being given access by the grant */
    context     table_entry  unique not null,
    /* References the actor record that inherits the role */
    target_type table_entry  unique,
    /* References the activity that triggered the Grant */
    fulfills    table_entry  unique
);

create table if not exists team
(
    uuid        uuid        primary key default gen_random_uuid(),
    id          text        unique not null,
    name        text        unique not null,
    content     text,
    summary     text,
    /* References the parent team(uuid) of this team */
    context     uuid        references team(uuid),
    /* References the activity inbox */
    inbox       uuid        unique not null references inbox(uuid),
    /* References the activity outbox */
    outbox      uuid        unique not null references outbox(uuid),
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
    role_filter uuid[],
    /* References to key(uuid) records for this team */
    key_ids     uuid[]
);

create table if not exists factory
(
    uuid                  uuid          primary key default gen_random_uuid(),
    id                    text          unique not null,
    name                  text          unique not null,
    available_actor_types actor_type[],
    inbox                 uuid          unique not null references inbox(uuid),
    outbox                uuid          unique not null references outbox(uuid),
    /* References to collaborator(uuid) records for this factory */
    collaborators         uuid[],
    /* References to follower(uuid) records for this factory */
    followers             uuid[],
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
    inbox     uuid    unique not null references inbox(uuid),
    outbox    uuid    unique not null references outbox(uuid),
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
    inbox     uuid    unique not null references inbox(uuid),
    outbox    uuid    unique not null references outbox(uuid),
    /* References to follower(uuid) records for this ticket tracker */
    followers uuid[],
    /* References to key(uuid) records for this ticket tracker */
    key_ids   uuid[]
);

create table if not exists repository
(
    uuid                  uuid    primary key default gen_random_uuid(),
    id                    text    unique not null,
    name                  text    unique not null,
    inbox                 uuid    unique not null references inbox(uuid),
    outbox                uuid    unique not null references outbox(uuid),
    /* List of URIs for cloning the repo */
    clone_uris            text[],
    /* List of URIs for pushing to the repo */
    push_uris             text[],
    /* References to repository(uuid) forks of this repo */
    forks                 uuid[],
    /* References to like(uuid) records for this repo */
    likes                 uuid[],
    /* References to follower(uuid) records for this repo */
    followers             uuid[],
    /* References to key(uuid) records for this repo */
    key_ids               uuid[],
    patches_tracked_by    uuid    references patch_tracker(uuid),
    tickets_tracked_by    uuid    references ticket_tracker(uuid),
    is_archived           bool,
    moved_to              uuid    references repository(uuid),
    mirrors               uuid    references repository(uuid),
    team                  uuid    unique references team(uuid)
);

create table if not exists person
(
    uuid         uuid    primary key default gen_random_uuid(),
    id           text    unique not null,
    name         text    unique not null,
    content      text,
    summary      text,
    /* References the activity inbox */
    inbox        uuid    unique not null references inbox(uuid),
    /* References the activity outbox */
    outbox       uuid    unique not null references outbox(uuid),
    /* References to follower(uuid) records for this person */
    followers    uuid[],
    /* References to key(uuid) records for this person */
    key_ids      uuid[]
);
