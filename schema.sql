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
