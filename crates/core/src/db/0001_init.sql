CREATE TABLE lookup_values (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('org_type', 'function', 'membership_category', 'cantonal_association')),
    name TEXT NOT NULL,
    sort INTEGER NOT NULL DEFAULT 0,
    UNIQUE (kind, name)
);

CREATE TABLE label_groups (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    applies_to TEXT NOT NULL CHECK (applies_to IN ('person', 'organisation', 'both'))
);

CREATE TABLE labels (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES label_groups (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    UNIQUE (group_id, name)
);

CREATE TABLE organisations (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    addition TEXT,
    street TEXT,
    po_box TEXT,
    zip TEXT,
    city TEXT,
    phone TEXT,
    email TEXT,
    website TEXT,
    branches INTEGER,
    apprentice_count INTEGER,
    notes TEXT,
    legacy_id TEXT,
    org_type_id INTEGER REFERENCES lookup_values (id),
    cantonal_association_id INTEGER REFERENCES lookup_values (id),
    search_text TEXT NOT NULL
);

CREATE TABLE persons (
    id INTEGER PRIMARY KEY,
    salutation TEXT,
    first_name TEXT,
    last_name TEXT NOT NULL,
    street TEXT,
    zip TEXT,
    city TEXT,
    phone TEXT,
    mobile TEXT,
    email TEXT,
    compensation TEXT,
    notes TEXT,
    search_text TEXT NOT NULL
);

CREATE TABLE affiliations (
    id INTEGER PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES persons (id) ON DELETE CASCADE,
    organisation_id INTEGER NOT NULL REFERENCES organisations (id) ON DELETE CASCADE,
    function_id INTEGER NOT NULL REFERENCES lookup_values (id),
    valid_from TEXT,
    valid_to TEXT
);
CREATE INDEX idx_affiliations_person ON affiliations (person_id);
CREATE INDEX idx_affiliations_org ON affiliations (organisation_id);

CREATE TABLE memberships (
    id INTEGER PRIMARY KEY,
    organisation_id INTEGER REFERENCES organisations (id) ON DELETE CASCADE,
    person_id INTEGER REFERENCES persons (id) ON DELETE CASCADE,
    category_id INTEGER REFERENCES lookup_values (id),
    joined_on TEXT,
    left_on TEXT,
    CHECK ((organisation_id IS NULL) <> (person_id IS NULL))
);

CREATE TABLE label_assignments (
    id INTEGER PRIMARY KEY,
    label_id INTEGER NOT NULL REFERENCES labels (id) ON DELETE CASCADE,
    organisation_id INTEGER REFERENCES organisations (id) ON DELETE CASCADE,
    person_id INTEGER REFERENCES persons (id) ON DELETE CASCADE,
    assigned_on TEXT,
    note TEXT,
    CHECK ((organisation_id IS NULL) <> (person_id IS NULL))
);

CREATE TABLE teams (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE team_members (
    id INTEGER PRIMARY KEY,
    team_id INTEGER NOT NULL REFERENCES teams (id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES persons (id) ON DELETE CASCADE,
    role TEXT,
    UNIQUE (team_id, person_id)
);

CREATE TABLE saved_groups (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    filter_json TEXT NOT NULL
);
