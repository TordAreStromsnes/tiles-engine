# Runtime Layer State Actions MVP

Runtime layer actions let gameplay triggers change map layer state without editing the map file itself. The first two actions are:

- `setLayerVisibility`: sets a layer visible or hidden.
- `setLayerOpacity`: sets a layer opacity between `0.0` and `1.0`.

Both actions target a `mapId` and `layerId`, and both carry a persistence mode.

## Persistence Modes

- `temporary`: applies during immediate gameplay and can be reset to map defaults. This is useful for roof fade, tunnel reveal, or short-lived fog.
- `session`: applies for the current runtime session and survives temporary resets.
- `persistent`: applies now and records pending save-delta intent. The actual save-delta file write is a follow-up.

The runtime state keeps `layerStates` for active overrides and `pendingPersistentLayerChanges` for future save integration.

## Sample Scenario

`sample_roof_entry_layer_actions` demonstrates a roof/tunnel style interaction: fade the `decor` layer temporarily and hide the `overlays` layer for the session. This keeps the mechanic generic enough for caves, houses, roof interiors, and hidden areas.
