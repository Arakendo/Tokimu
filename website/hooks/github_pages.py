from pathlib import Path


def on_post_build(config, **kwargs):
    """Disable GitHub Pages Jekyll processing for the generated MkDocs site."""
    site_dir = Path(config["site_dir"])
    (site_dir / ".nojekyll").touch()
