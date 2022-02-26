# Bevy Game Jam Game

## Assets

Get the assets files from the separate [`bevyjam-assets-dist` repository][https://github.com/PapugaPower/bevyjam-assets-dist].

During development, you may find it convenient to create a filesystem link
(symlink on unix, or directory junction on windows) to the `assets` directory
from that repo.

Unix:

```
ln -sfr /path/to/bevyjam-assets-dist/assets /path/to/bevyjam/assets
```

Windows (cmd):

```
MKLINK /J C:\Path\To\bevyjam\assets C:\Path\To\bevyjam-assets-dist\assets
```
