import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
html=(ROOT/'static'/'index.html').read_text(encoding='utf-8')
tables=json.loads((ROOT/'ui'/'data'/'tables.json').read_text(encoding='utf-8'))
def test_provider_select_is_short():
    chunk=html.split('id="cfg-provider"')[1].split('</select>')[0]
    assert chunk.count('<option value=')==14 and chunk.count('<optgroup')==1
    assert 'value="amni"' in chunk and 'value="xai"' in chunk and 'value="ollama"' in chunk
    for junk in ('deepseek','groq','mistral','cohere','together','fireworks','perplexity','openrouter'):
        assert f'value="{junk}"' not in chunk
def test_adam_sizes_and_links():
    assert tables['core_providers']['amni']['models']==['adam:3b','adam:8b']
    assert tables['model_labels']['adam:3b'].startswith('Adam 3B')
    assert tables['model_labels']['adam:8b'].startswith('Adam 8B')
    assert 'https://amni-scient.com"' in html
    assert 'https://amni-scient.com/amni-code"' in html
if __name__=='__main__':
    test_provider_select_is_short()
    test_adam_sizes_and_links()
    print('ok')
