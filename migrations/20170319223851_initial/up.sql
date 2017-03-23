create table pastes (
    id            serial PRIMARY KEY,
    key           uuid UNIQUE NOT NULL,
    content       text NOT NULL,
    date_created  timestamp WITH TIME ZONE NOT NULL DEFAULT NOW(),
    date_viewed   timestamp WITH TIME ZONE NOT NULL DEFAULT NOW()
);

