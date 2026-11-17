import re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
html=(ROOT/'static'/'index.html').read_text(encoding='utf-8')
start=(ROOT/'static'/'start.js').read_text(encoding='utf-8')
settings=(ROOT/'ui'/'settings.he').read_text(encoding='utf-8')
def test_skin_and_theme_before_first_paint():
    head=html.split('</head>')[0]
    assert '<script src="/assets/start.js"></script>' in head
    assert 'localStorage.getItem(k)' in start and "g('amni_skin')" in start and "g('amni-theme')" in start
    assert start.index("setAttribute('data-skin'")<start.index("import('/assets/helium.js')")
    assert start.index("setAttribute('data-theme'")<start.index("import('/assets/helium.js')")
    assert not re.search(r'<script>[^<]',html)
def test_cwd_set_after_config():
    assert 'id="cwd-path"' in html
    assert '"/api/config"' in settings and '"working_dir"' in settings and 'web::el("cwd-path")' in settings
if __name__=='__main__':
    test_skin_and_theme_before_first_paint()
    test_cwd_set_after_config()
    print('ok')
