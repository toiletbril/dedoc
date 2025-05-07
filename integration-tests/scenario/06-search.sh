#!/bin/sh

# See if dedoc can search downloaded docsets.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc rm --purge-all

mock_dedoc dl docset-3~1

# Search for files.
mock_dedoc ss docset-3~1 | mock_diff_stdin_to_text \
'Searching for ``...
Exact matches in `docset-3~1`:
   1  commands/vacuum_full
   2  concepts/bloat_storage
   3  concepts/compile_time
   4  concepts/null_problems
   5  concepts/orm_cringe
   6  errors/commit_failed
   7  errors/e_mom_yelling
   8  functions/e_borrow_checker
   9  guides/e_lifetime_errors
  10  guides/mayonnaise_benchmarks
  11  indexes/waifu_index
  12  tables/e_anime_schema
        13  #index-recommendations
        14  #schema-definition'

mock_dedoc ss docset-3~1 errors | mock_diff_stdin_to_text \
'Searching for `errors`...
Exact matches in `docset-3~1`:
   1  errors/commit_failed
   2  errors/e_mom_yelling
   3  guides/e_lifetime_errors'

# Ignore case.
mock_dedoc ss docset-3~1 -i ERRORS | mock_diff_stdin_to_text \
'Searching for `ERRORS`...
Exact matches in `docset-3~1`:
   1  errors/commit_failed
   2  errors/e_mom_yelling
   3  guides/e_lifetime_errors'

mock_dedoc ss docset-3~1 -p startup | mock_diff_stdin_to_text \
"Searching for \`startup\`...
No exact matches in \`docset-3~1\`.
Mentions in other files from \`docset-3~1\`:
   1  errors/e_mom_yelling
        ...'><h3>Option 2: \"Im working on a startup!\"</h3><p>Buys you 3-6 months</p>..."

mock_dedoc ss docset-3~1 -pw you | mock_diff_stdin_to_text \
'Searching for ` you `...
No exact matches in `docset-3~1`.
Mentions in other files from `docset-3~1`:
   1  errors/e_mom_yelling
        ...orking on a startup!"</h3><p>Buys you 3-6 months</p></div></section></d...'

# Test if dedoc creates search cache.
mock_dedoc ss docset-3~1 -o 7
mock_cat "$DEDOC_HOME/search_cache.json" | mock_diff_stdin_to_text \
'```
{"exact_results":[{"item":"commands/vacuum_full","fragment":null},{"item":"concepts/bloat_storage","fragment":null},{"item":"concepts/compile_time","fragment":null},{"item":"concepts/null_problems","fragment":null},{"item":"concepts/orm_cringe","fragment":null},{"item":"errors/commit_failed","fragment":null},{"item":"errors/e_mom_yelling","fragment":null},{"item":"functions/e_borrow_checker","fragment":null},{"item":"guides/e_lifetime_errors","fragment":null},{"item":"guides/mayonnaise_benchmarks","fragment":null},{"item":"indexes/waifu_index","fragment":null},{"item":"tables/e_anime_schema","fragment":null},{"item":"tables/e_anime_schema","fragment":"index-recommendations"},{"item":"tables/e_anime_schema","fragment":"schema-definition"}],"vague_results":[]}
```'
mock_cat "$DEDOC_HOME/search_cache_options.json" | mock_diff_stdin_to_text \
'```
{"query":"","docset":"docset-3~1","options":{"case_insensitive":false,"precise":false,"whole":false}}
```'

# Open some pages.
mock_dedoc ss docset-3~1 -o 7 | mock_diff_stdin_to_text \
'# ERROR: Mom Yelling (╬ Ò﹏Ó)

ERROR CODE: 0xDEADBEEF                                                          
SEVERITY: Maximum volume                                                        
LOCATION: Basement                                                              
QUERY: SELECT * FROM real_world WHERE responsibility = true;                    

## Suggested Fixes

### Option 1: Pretend Not To Hear

Works until she unplugs the router

### Option 2: "Im working on a startup!"

Buys you 3-6 months'


mock_dedoc ss docset-3~1 -o 12 | mock_diff_stdin_to_text \
"# Anime Database Schema (￣ω￣;)

───────────┬──────┬───────────────────────────
Column     │Type  │Description                
───────────┼──────┼───────────────────────────
best_waifu │TEXT  │Objectively correct opinion
───────────┼──────┼───────────────────────────
power_level│BIGINT│Always over 9000           
───────────┴──────┴───────────────────────────

## Recommended Indexes
* CREATE INDEX idx_tsundere_rage ON waifus (anger_level) WHERE dere_type =
  'tsundere'"

# Query fragments. Command below triggers an error on old versions of html2text.
mock_dedoc ss docset-3~1 -o 13 | mock_diff_stdin_to_text \
"...
## Recommended Indexes
* CREATE INDEX idx_tsundere_rage ON waifus (anger_level) WHERE dere_type =
  'tsundere'"

mock_dedoc ss docset-3~1 -o 14 | mock_diff_stdin_to_text \
'...
Column     │Type  │Description                
───────────┼──────┼───────────────────────────
best_waifu │TEXT  │Objectively correct opinion
...'

# Ignore the fragment. Will be the same as `-o 12`.
mock_dedoc ss docset-3~1 -o 14 -f | mock_diff_stdin_to_text \
"# Anime Database Schema (￣ω￣;)

───────────┬──────┬───────────────────────────
Column     │Type  │Description                
───────────┼──────┼───────────────────────────
best_waifu │TEXT  │Objectively correct opinion
───────────┼──────┼───────────────────────────
power_level│BIGINT│Always over 9000           
───────────┴──────┴───────────────────────────

## Recommended Indexes
* CREATE INDEX idx_tsundere_rage ON waifus (anger_level) WHERE dere_type =
  'tsundere'"
