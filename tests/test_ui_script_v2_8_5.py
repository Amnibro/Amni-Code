import re
from pathlib import Path
html=(Path(__file__).resolve().parents[1]/'static'/'index.html').read_text(encoding='utf-8')
def test_theme_if_not_split_by_fetch():
    assert not re.search(r"if\(window\.AmniSkin\)\{[^}]*\}\s*fetch\([^)]*\)[^;]*;\s*else if\(savedTheme",html)
    assert re.search(r"if\(window\.AmniSkin\)\{[^}]*\}\s*else if\(savedTheme",html)
def test_cwd_set_after_config():
    assert "cwd-path" in html and "working_dir" in html
    assert "loadConfig();" in html
if __name__=='__main__':
    test_theme_if_not_split_by_fetch()
    test_cwd_set_after_config()
    print('ok')
