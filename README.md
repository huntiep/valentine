# Valentine

A Git server for small-medium sized projects that want to self-host.

### Install
```bash
createdb valentine
adduser git
cp -r valentine /home/git
su git
cd ~/valentine
./valentine web
```

### TODO
  - API - EASY
    - Would need API tokens to be added
  - Webhooks - MEDIUM
  - OTP - MEDIUM
    - Git servers are pretty important, so supporting OTP would be good
  - Test with large repositories
  - Replace use of git command
  - Replace git2 with valgit?
  - More git info
    - Maybe show what refs point to a commit in commit and log views
    - Maybe show diffs on commit view
  - Permissions
    - Right now only the owner of a repo can push to it, and only the owner can
    do anything with private repos.
    - We need a way to give users permissions on repositories
      - One way to do this might be a "write_permissions" field for repos which
      contains a list of users with write access
      - For private repos we also need read/view permission
    - Most places with permission checks have been marked with TODO and the checks
    will need to be changed. The new check should be straightforward
  - Determine git workflow
    - Valentine just handles displaying repo data, but it should have the machinery
    to work with tools for a git workflow (bug tracker, code review, etc.)
    - The fork/pull request model will not work for valentine
    - Maybe look at how gerrit works and base something on that?
