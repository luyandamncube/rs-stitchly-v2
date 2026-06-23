, raw as (
    select
      trim(coalesce(stdout, '')) as stdout_text,
      coalesce(stderr, '') as stderr_text,
      try_cast(exit_code as integer) as shell_exit_code,
      try_cast(duration_ms as bigint) as shell_duration_ms
    from input
  ),
  lines as (
    select
      row_number() over () as line_number,
      trim(line) as stdout_line
    from raw,
      unnest(string_split(stdout_text, chr(10))) as t(line)
    where trim(line) <> ''
  ),
  parsed as (
    select
      *,
      (
        select try_cast(stdout_line as json)
        from lines
        where try_cast(stdout_line as json) is not null
        order by line_number desc
        limit 1
      ) as payload
    from raw
  ),
  fields as (
    select
      json_extract_string(payload, '$.repo_key') as repo_key,
      json_extract_string(payload, '$.remote_url') as remote_url,
      json_extract_string(payload, '$.branch') as branch,
      json_extract_string(payload, '$.repo_path') as repo_path,
      coalesce(json_extract_string(payload, '$.previous_commit'), '') as previous_commit,
      coalesce(json_extract_string(payload, '$.head_commit'), '') as head_commit,
      coalesce(
        try_cast(json_extract_string(payload, '$.should_skip') as boolean),
        false
      ) as should_skip,
      shell_exit_code,
      shell_duration_ms,
      stdout_text as raw_stdout,
      stderr_text as raw_stderr,
      payload is not null as parsed_ok
    from parsed
  )
  select
    *,
    coalesce(shell_exit_code, -1) = 0 and parsed_ok as sync_ok,
    case
      when coalesce(shell_exit_code, -1) <> 0 then 'shell_failed'
      when not parsed_ok then 'parse_failed'
      when should_skip then 'unchanged'
      else 'changed'
    end as sync_status
  from fields
