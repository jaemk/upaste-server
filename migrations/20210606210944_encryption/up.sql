begin;

alter table pastes
    add column nonce text;
alter table pastes
    add column salt text;
alter table pastes
    add column signature text;

commit;
