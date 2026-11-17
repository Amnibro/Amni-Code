import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
html=(ROOT/'static'/'index.html').read_text(encoding='utf-8')
tables=(ROOT/'ui'/'data'/'tables.json').read_text(encoding='utf-8')
def test_provider_select_is_short():
    chunk=html.split('id="cfg-provider"')[1].split('</select>')[0]
    assert chunk.count('<option value=')==14 and chunk.count('<optgroup')==1
    assert 'value="amni"' in chunk and 'value="xai"' in chunk and 'value="ollama"' in chunk
    for junk in ('deepseek','groq','mistral','cohere','together','fireworks','perplexity','openrouter'):
        assert f'value="{junk}"' not in chunk
def test_model_catalog_is_short():
    assert len(json.loads(tables)['core_providers'])==14
    for src in (html,tables):
        assert 'gpt-5' not in src
        assert 'mixtral-8x7b-32768' not in src
    assert 'adam:granite-gf17' in tables
if __name__=='__main__':
    test_provider_select_is_short()
    test_model_catalog_is_short()
    print('ok')
