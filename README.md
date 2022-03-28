# MultiSQL
Diverged from [GlueSQL](https://github.com/gluesql/gluesql) as of [GlueSQLv0.5.0](https://github.com/gluesql/gluesql/releases/tag/v0.5.0).

See origin differences at [#8](https://github.com/SyRis-Consulting/gluesql/pull/8).

Main *TODO*s:
- [x] `TRUNCATE`
- [x] Variables
- [x] `WITH`
- [x] Indexing
- [x] More optimisations
	- [ ] Even more optimisations
- [ ] Multi Database everything
	- [x] `SELECT`
	- [ ] `INSERT`
	- [ ] `UPDATE`
	- [ ] `DELETE`
	- [ ] `CREATE`
	- [ ] `DROP`
- [ ] `FIRST`/`LAST`
- [ ] `EXECUTE`
- [ ] `IN`
- [ ] Subqueries
- [ ] `TIMESTAMP` datatype
- [ ] `INTERVAL` datatype
- [ ] Memory storage
- [ ] XML+ZIP (Excel and such) storage
- [ ] Primary Key
- [ ] Foreign Key
- [ ] Optional multithreading
- [ ] Clean up bad code (`clone`s and grossness)
- [ ] More error information/context
	- (without the cost of perf somehow?)
- [ ] Fix unreachable error areas
- [ ] Testing

Eventually:
- [ ] Config
	- [ ] Permissions
- [ ] Pre plan (allows choice and analysis of specific optimisations and such)
- [ ] Transaction log
- [ ] Transaction store
- [ ] Query undo
