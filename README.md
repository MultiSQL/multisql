# MultiSQL
[![Rust](https://github.com/KyGost/multisql/actions/workflows/rust.yml/badge.svg)](https://github.com/KyGost/multisql/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/KyGost/multisql/branch/main/graph/badge.svg?token=RX0OCX7AJ6)](https://codecov.io/gh/KyGost/multisql)
[![Chat](https://img.shields.io/discord/780298017940176946)](https://discord.gg/C6TDEgzDzY)
[![LICENSE](https://img.shields.io/crates/l/gluesql.svg)](https://github.com/gluesql/gluesql/blob/main/LICENSE)

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
- [ ] Clean up bad code (`clone`s and grossness)
- [ ] More error information/context
	- (without the cost of perf somehow?)
- [ ] Fix unreachable error areas
- [ ] Testing
- [ ] Clippy

Eventually:
- [ ] Config
	- [ ] Permissions
- [ ] Pre plan (allows choice and analysis of specific optimisations and such)
- [ ] Transaction log
- [ ] Transaction store
- [ ] Query undo
