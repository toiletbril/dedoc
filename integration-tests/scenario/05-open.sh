#!/bin/sh

# See if dedoc can open things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc dl docset-1

# Open a page from some docset.
mock_dedoc open docset-1 type-1/1 | mock_diff_stdin_to_text "# test"

# Check whether dedoc can act as a HTML transpiler. Default width is 80
# characters.
mock_dedoc open --html "./data/example-page-mayonnaise.html" | \
mock_diff_stdin_to_text \
'# postgresql confessed its love to rust but got a compile-time error: "lifetime
# forever too short" (´；ω；｀)

rust is that partner who makes you wear a memory-safe helmet during pillow talk
(￣ヘ￣;)✧

## my mom keeps yelling "WHEN WILL YOU GET A REAL JOB" while im elbow-deep in
## mayonnaise benchmarks (╬ Ò﹏Ó)

accidentally implemented anime recommendations as a postgres extension... again
(；一_一)
* postgresqls transaction isolation levels describe my dating history (´･_･`)
* my linkedin: "senior anime binger with minor in deadlock mediation" (⌐■_■)
* mayonnaise is the only blob storage that brings me joy (づ￣ ³￣)づ
1. swear this time ill learn proper database normalization (•̀o•́)ง
2. get distracted by new isekai trash instead (ノಠ益ಠ)ノ
3. sob into mayonnaise jar while postgres vacuum runs for 6 hours (╥﹏╥)

> "this database schema is more chaotic than /b/ during a server meltdown" ┻━┻
> ︵ヽ(`Д´)ﾉ︵ ┻━┻

click here to discover which sql injection vulnerability matches your zodiac <#>
sign (◔_◔) <#> my postgresql performance tuning notes are just anime doodles

────────────────────────────────────────────┬───────────────────────────────────
your love life has worse indexing than my   │rusts error messages are longer    
test database (눈_눈)                       │than my list of regrets (╯︵╰,)    
────────────────────────────────────────────┼───────────────────────────────────
"working remotely" means watching anime     │my postgres config has more issues 
while cargo downloads half of crates.io     │than my therapists notepad (；´∀｀)
(￢_￢;)                                    │                                   
────────────────────────────────────────────┴───────────────────────────────────'

# Use 66 columns and -n.
mock_dedoc open --html "./data/example-page-mayonnaise.html" -c 66 -n | \
mock_diff_stdin_to_text \
'    1  # postgresql confessed its love to rust but got a
    2  # compile-time error: "lifetime forever too short"
    3  # (´；ω；｀)
    4  
    5  rust is that partner who makes you wear a memory-safe
    6  helmet during pillow talk (￣ヘ￣;)✧
    7  
    8  ## my mom keeps yelling "WHEN WILL YOU GET A REAL JOB"
    9  ## while im elbow-deep in mayonnaise benchmarks (╬ Ò﹏Ó)
   10  
   11  accidentally implemented anime recommendations as a
   12  postgres extension... again (；一_一)
   13  * postgresqls transaction isolation levels describe my
   14    dating history (´･_･`)
   15  * my linkedin: "senior anime binger with minor in deadlock
   16    mediation" (⌐■_■)
   17  * mayonnaise is the only blob storage that brings me joy
   18    (づ￣ ³￣)づ
   19  1. swear this time ill learn proper database normalization
   20     (•̀o•́)ง
   21  2. get distracted by new isekai trash instead (ノಠ益ಠ)ノ
   22  3. sob into mayonnaise jar while postgres vacuum runs for 6
   23     hours (╥﹏╥)
   24  
   25  > "this database schema is more chaotic than /b/ during a
   26  > server meltdown" ┻━┻ ︵ヽ(`Д´)ﾉ︵ ┻━┻
   27  
   28  click here to discover which sql injection vulnerability <#>
   29  matches your zodiac sign (◔_◔) <#> my postgresql performance
   30  tuning notes are just anime doodles
   31  
   32  ────────────────────────────────┬──────────────────────────
   33  your love life has worse        │rusts error messages are  
   34  indexing than my test database  │longer than my list of    
   35  (눈_눈)                         │regrets (╯︵╰,)           
   36  ────────────────────────────────┼──────────────────────────
   37  "working remotely" means        │my postgres config has    
   38  watching anime while cargo      │more issues than my       
   39  downloads half of crates.io     │therapists notepad        
   40  (￢_￢;)                        │(；´∀｀)                  
   41  ────────────────────────────────┴──────────────────────────'
