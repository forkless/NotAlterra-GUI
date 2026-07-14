# BAFH Diary

BAFH-style chronicle of the most chaotic and memorable parts of each session.

## 2026-06-29 — The 8-Hour Reboot from Hell

- Spent 8 hours debugging "app not launching" when the REAL problem was `Start-Process` killing children on this machine
- User turned gray from pulling hair over a non-existent bug
- filter-repo wiped the entire docs/ directory (oops)
- Game Guard refused to appear in the tech specs card for 6 iterations — wrong Grid.Row placement
- Inno Setup logo had 6 different padding attempts before getting to 65x65 with 10px right/top margins
- License page had the wrong labels blanked — swapped LicenseLabel and LicenseLabel3 three times
- Conclusion: `cmd /c start "" "app.exe"` is the only way to launch on this machine. Remember it.
