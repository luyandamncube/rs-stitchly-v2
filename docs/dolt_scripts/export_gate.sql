select *
from input
where coalesce(sync_ok, false) = true
  and coalesce(should_skip, false) = false
