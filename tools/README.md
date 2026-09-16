# Rendering the marketplace preview

`preview.png` in the repository root is the marketplace listing image. It is
**rendered, not screenshotted**: the row layout and the helper functions come
from `Menu.qml`, and the data in it is invented.

That is deliberate. A screenshot of a working panel is a screenshot of
somebody's real machine — their session names, their project paths, whatever
window happens to be behind it. An earlier attempt at this captured a
client's dashboard and came within one `git commit` of publishing it.

To regenerate:

```bash
QT_QPA_PLATFORM=offscreen /usr/lib/qt6/bin/qml -I tools tools/preview.qml
```

It writes to the path named at the bottom of `tools/preview.qml`. The stub
`qs.Commons`/`qs.Ui` modules under `tools/qs` stand in for the Omarchy shell's
own, which cannot be loaded outside `quickshell`.
