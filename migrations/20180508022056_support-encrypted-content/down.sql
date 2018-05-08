pragma foreign_keys = off;

begin transaction;

alter table pastes rename to all_pastes;

create table pastes (
    id              integer primary key autoincrement,
    key             text unique not null,
    content         text not null,
    content_type    text not null default 'text',
    date_created    unsigned big int not null default (cast(strftime('%s', 'now') as unsigned big int)),
    date_viewed     unsigned big int not null default (cast(strftime('%s', 'now') as unsigned big int))
);

insert into pastes (key, content, content_type, date_created, date_viewed)
    select all_pastes.key, text_pastes.content, all_pastes.content_type,
           all_pastes.date_created, all_pastes.date_viewed
    from text_pastes
        inner join all_pastes on text_pastes.id=all_pastes.text_paste;

drop table text_pastes;
drop table blob_pastes;
drop table all_pastes;
drop table authorization;

pragma foreign_keys = on;

commit;
