create table pastes (
    id            serial PRIMARY KEY,
    key           text UNIQUE NOT NULL,
    content       text NOT NULL,
    content_type  text NOT NULL DEFAULT 'text',
    date_created  timestamp WITH TIME ZONE NOT NULL DEFAULT NOW(),
    date_viewed   timestamp WITH TIME ZONE NOT NULL DEFAULT NOW()
);

