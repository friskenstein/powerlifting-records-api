create table Records (
	id integer primary key,
	sex   text not null,
	div   text not null,
	event text not null,
	equip text not null,
	class text not null,
	lift  text not null,

	weight real default 0,
	name   text,
	date   text,
	place  text,

	unique(sex, div, event, equip, class, lift)
);

-- Build-time errors captured while parsing CSVs
create table Errors (
    id integer primary key,
    file  text not null,
    line  integer not null,
    key   text,
    weight real,
    name  text,
    date  text,
    place text,
    reason text not null
);
