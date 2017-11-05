create table pastes (
    id              integer PRIMARY KEY AUTOINCREMENT,
    key             text UNIQUE NOT NULL,
    content         text NOT NULL,
    content_type    text NOT NULL DEFAULT 'text',
    date_created    unsigned big int NOT NULL DEFAULT (cast(strftime('%s', 'now') as unsigned big int)),
    date_viewed     unsigned big int NOT NULL DEFAULT (cast(strftime('%s', 'now') as unsigned big int))
);

