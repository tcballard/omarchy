# Theme-controlled panel headers

A panel can identify its header through `headerItem` on `KeyboardPanel` or `PopupCard`. Lazy KeyboardPanel content may expose a `property alias headerItem: hero`; the host reads that alias after creation. No header is inferred from child order or text.

Themes opt in through `shell.popups.toml`:

```toml
[popups]
header-style = "joined"
header-background = "#e9edf2"
header-background-alpha = 1.0
```

Missing, `inset`, and unknown styles keep the existing panel rendering. Missing header colour inherits the popup background. Theme changes and machine overrides use the existing Color shell-value merge, so switching away from an opted-in theme resets the style. Older shells ignore these keys.

The joined fill uses the same rounded shape as the frame, clipped to a header band; the border is painted over it. The band ends half a panel padding below the header, leaving the existing body spacing intact. It leaves per-side borders and gradients intact, has no pointer handler, and does not move the header, its controls, body content, or focus targets. Header scrolling changes the visible fill height; hiding or destroying the header removes it. Audio, Network, Dropbox and Tailscale expose headers in this initial integration. Other panels and third-party plugins opt in by identifying their own header item; notification toasts do not have a panel header.

Familiar supplies the light grey header with dark existing text. This patch belongs to the shell and the separate theme repository, and does not depend on Familiar Desktop.

## Desktop verification

Use the shell and theme feature branches together. Record both commit SHAs, Omarchy/Qt versions and display scaling. Open Audio and Network, check their header switches/actions with mouse and keyboard, and verify the fill joins the outer corners without touching the border. Check Dropbox logged in/out and Tailscale installed/not installed. Test long text, large fonts, a short panel that scrolls, fractional scaling, and a per-side/gradient border. Switch to a theme without the new tokens and back, then override header-style with inset in the user's shell.toml. Check other popups and notification toasts retain their normal appearance.

Automated or isolated component checks do not establish live desktop acceptance. Keep the changes in draft until the actual panels are verified.

## Recorded validation

The isolated offscreen Quickshell 0.3.2 render on Qt 6.12.0 passed twelve runtime assertions. The rendered inset/joined comparison was inspected for corner joins, body spacing and clipping. This fixture does not load the live Audio/Network/Dropbox/Tailscale services or prove their desktop interaction; those remain pending.
