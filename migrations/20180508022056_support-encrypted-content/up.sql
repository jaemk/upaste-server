pragma foreign_keys = on;

begin transaction;

alter table pastes rename to text_pastes_old;

create table text_pastes (
    id         integer primary key autoincrement,
    content    text not null
);

create table blob_pastes (
    id         integer primary key autoincrement,
    iv         blob not null,
    content    blob not null
);

create table authorization (
    id              integer primary key autoincrement,
    salt            blob not null,
    hash            blob not null,
    date_created    unsigned big int not null default (cast(strftime('%s', 'now') as unsigned big int))
);

create table pastes (
    id              integer primary key autoincrement,
    key             text unique not null,
    content_type    text not null default 'text',
    text_paste      integer,
    blob_paste      integer,
    auth            integer,
    date_created    unsigned big int not null default (cast(strftime('%s', 'now') as unsigned big int)),
    date_viewed     unsigned big int not null default (cast(strftime('%s', 'now') as unsigned big int)),
    foreign key(text_paste) references text_pastes(id),
    foreign key(blob_paste) references blob_pastes(id),
    foreign key(auth) references authorization(id)
);

insert into text_pastes (id, content)
    select id, content
    from text_pastes_old;

insert into pastes (key, content_type, text_paste, date_created, date_viewed)
    select text_pastes_old.key, text_pastes_old.content_type, text_pastes_old.id,
           text_pastes_old.date_created, text_pastes_old.date_viewed
    from text_pastes_old;

drop table text_pastes_old;

commit;

