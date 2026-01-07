# VRM Model Directory

Place your VRM 1.0 model files in this directory to be loaded by Bevy's asset server.

## Getting VRM Models

You can obtain VRM models from:

- [VRoid Hub](https://hub.vroid.com/) - Community-created VRM avatars
- [VRoid Studio](https://vroid.com/studio) - Create your own VRM avatars
- Other VRM-compatible tools and platforms

## Default Model

By default, the application loads the model from the asset path `vrm/model.vrm` (relative to the `assets/` directory). You can either:

1. Place your VRM file here and name it `model.vrm`
2. Modify the code in `src/main.rs` to load a different asset path

**Note**: Models must be in the `assets/` directory or its subdirectories to be accessible by Bevy's asset server. You cannot load VRM files from arbitrary filesystem paths.

## File Format

This application only supports **VRM 1.0** format files (`.vrm` extension).

## License Note

VRM models may have various licenses. Please respect the creator's license terms when using VRM models.
