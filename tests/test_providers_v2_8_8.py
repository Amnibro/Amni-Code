from pathlib import Path
html=(Path(__file__).resolve().parents[1]/'static'/'index.html').read_text(encoding='utf-8')
def test_provider_select_is_short():
    chunk=html.split('id="cfg-provider"')[1].split('</select>')[0]
    assert chunk.count('<option value=')==14 and chunk.count('<optgroup')==1
    assert 'value="amni"' in chunk and 'value="xai"' in chunk and 'value="ollama"' in chunk
    for junk in ('deepseek','groq','mistral','cohere','together','fireworks','perplexity','openrouter'):
        assert f'value="{junk}"' not in chunk
def test_adam_sizes_and_links():
    assert "models:['adam:3b','adam:8b']" in html
    assert "'adam:3b':'Adam 3B" in html
    assert "'adam:8b':'Adam 8B" in html
    assert 'https://amni-scient.com"' in html
    assert 'https://amni-scient.com/amni-code"' in html
    assert 'https://ko-fi.com/amnibro"' in html
    assert 'class="hdr-links"' in html
    assert 'gpt-5' not in html
    assert 'mixtral-8x7b-32768' not in html
if __name__=='__main__':
    test_provider_select_is_short()
    test_adam_sizes_and_links()
    print('ok')
