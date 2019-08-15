-- Fix millisecond time stamps that have been generated by an early version until 2018-01-19 (1516382882521)
update categories set created=created/1000 where created>1000000000000;
update comments set created=created/1000 where created>1000000000000;
update comments set archived=archived/1000 where archived not null and archived>1000000000000;
update entries set created=created/1000 where created>1000000000000;
update entries set archived=archived/1000 where archived not null and archived>1000000000000;
update events set archived=archived/1000 where archived not null and archived>1000000000000;
update ratings set created=created/1000 where created>1000000000000;
update ratings set archived=archived/1000 where archived not null and archived>1000000000000;

-- Delete invalid tags with less than 2 characters (= empty or single character)
delete from entry_tag_relations where length(tag_id) < 2;
delete from event_tag_relations where length(tag_id) < 2;
delete from org_tag_relations where length(tag_id) < 2;
delete from tags where length(id) < 2;

-- Delete orphaned tag relations
delete from entry_tag_relations where (entry_id, entry_version) not in (select id, version from entries);
delete from event_tag_relations where event_id not in (select id from events);
delete from org_tag_relations where org_id not in (select id from organizations);

-- Insert missing tags (just in case some got lost along the way)
insert or ignore into tags (id) select tag_id from entry_tag_relations;
insert or ignore into tags (id) select tag_id from event_tag_relations;
insert or ignore into tags (id) select tag_id from org_tag_relations;
