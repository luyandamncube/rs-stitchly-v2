, checked as (
    select
      case
        when coalesce(sync_ok, false) = false then
          error('Dolt sync failed or produced invalid metadata')
        else 0
      end as _sync_assert,
      *
    from input
  )
  select * exclude (_sync_assert)
  from checked
