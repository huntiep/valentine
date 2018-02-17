# Valentine

A Git server aimed at small - medium sized projects that want to self-host.

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
  - CSS - EASY
  - API - EASY
    - Would need API tokens to be added
  - Webhooks - MEDIUM
  - OTP - MEDIUM
    - Git servers are pretty important, so supporting OTP would be good
