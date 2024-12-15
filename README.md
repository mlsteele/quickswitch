# QUICKSWITCH

Switch apps using the keyboard.

## Which apps?
See main.rs for the active rules. Search for "Firefox" for an example.

## Launchd
To run as a background service on macos use the `launchd.plist`.

```
sudo touch /var/log/quickswitch.log /var/log/quickswitch.err.log
sudo chmod 666 /var/log/quickswitch.err.log /var/log/quickswitch.log
launchctl load -w launchd.plist
launchctl list | grep quickswitch
launchctl start com.milessteele.quickswitch
```

Add quickswitch as as able app in Settings > Privacy & Security > Accessibility.
