# MultiSQL
Diverged from [GlueSQL](https://github.com/gluesql/gluesql) as of [GlueSQLv0.5.0](https://github.com/gluesql/gluesql/releases/tag/v0.5.0).

See differences core/origin differences at [#8](https://github.com/SyRis-Consulting/gluesql/pull/8).

Main *TODO*s:
- [ ] `TRUNCATE`
- [ ] `FIRST`/`LAST`
- [ ] `EXECUTE`
- [x] Variables
- [ ] `WITH`
- [ ] `IN`
- [ ] Subqueries
- [ ] `TIMESTAMP` datatype
- [ ] `INTERVAL` datatype
- [ ] Testing
- [ ] Fix unreachable error areas
- [ ] More error information/context
	- (without the cost of perf somehow?)
- [ ] More optimisations
- [ ] Clean up bad code (`clone`s and grossness)
- [ ] Optional multithreading
- [ ] Memory storage
- [ ] XML+ZIP (Excel and such) storage
- [ ] Multi Database everything
	[x] - `SELECT`
	- [ ] `INSERT`
	- [ ] `UPDATE`
	- [ ] `DELETE`
	- [ ] `CREATE`
	- [ ] `DROP`

Eventually:
- [ ] Config
	- [ ] Permissions
- [ ] Pre plan (allows choice and analysis of specific optimisations and such)
