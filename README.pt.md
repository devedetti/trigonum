# Trigonum

*[Español](README.md) · [English](README.en.md) · Português*

> *trigonum*: “triângulo” em latim. Um arquivo STL não passa de uma lista de triângulos.

Visualizador rápido e leve de modelos 3D em formato STL para Linux, feito com GTK 4 e OpenGL.

O repositório original fica em [git.local.net.ar/nachopro/trigonum](https://git.local.net.ar/nachopro/trigonum);
o [GitHub](https://github.com/devedetti/trigonum) é um espelho. Issues e pull requests vão no GitHub.

## Por que eu fiz isto

Faz algumas semanas que venho desenhando modelos 3D no OpenSCAD, que depois um vizinho
imprime para mim. Toda vez que eu exportava um STL e queria ver se tinha ficado bom,
precisava abrir em algum site: lento, pesado, cheio de propaganda, com o modelo travando
toda vez que eu tentava girar. Para algo que você faz de novo e de novo durante o dia,
era um martírio.

Então fiz meu próprio visualizador. Agora eu abro o arquivo e ele aparece na hora, gira
lisinho, e quando salvo no OpenSCAD ele se atualiza sozinho. Roda na velocidade da luz e
eu estou muito feliz.

Por trás disso tudo há uma convicção: a web é para páginas, não para programas. Ela é
ótima para o internet banking ou para ler um blog, mas quando você precisa trabalhar de
verdade, JavaScript não é opção: tudo fica lento, aparecem os travamentos e ainda por cima
você tem que aguentar os anúncios e toda a tralha que vem de brinde. Um programa de desktop
bem feito abre rápido, usa o que precisa usar e não pede nada em troca.

Que voltem os apps de desktop.

## Recursos

- Arquivos STL binários e ASCII, inclusive modelos com milhões de triângulos
- Dimensões, quantidade de triângulos, volume e área de superfície
- Abas: vários modelos abertos ao mesmo tempo, cada um com sua vista
- Malha de arame sobreposta
- Recarga automática quando o arquivo muda no disco (ideal para trabalhar com OpenSCAD ou exportações de CAD)
- Arrastar e soltar, e abertura pelo gerenciador de arquivos
- Idiomas: português, espanhol e inglês

## Controles

| Ação | Mouse / tecla |
|---|---|
| Orbitar | Botão esquerdo + arrastar |
| Deslocar | Botão do meio ou direito + arrastar, ou Shift + botão esquerdo |
| Zoom | Roda do mouse |
| Enquadrar o modelo | Duplo clique ou `F` |
| Malha de arame | `W` |
| Painel de informações | `I` ou `F9` |
| Abrir / Recarregar | `Ctrl+O` / `Ctrl+R` ou `F5` |
| Fechar aba | `Ctrl+W` ou clique do meio na aba |
| Próxima aba / aba anterior | `Ctrl+Tab` / `Ctrl+Shift+Tab`, ou `Ctrl+PgDn` / `Ctrl+PgUp` |
| Ir para a aba 1…8 / para a última | `Alt+1`…`Alt+8` / `Alt+9` |
| Sair | `Ctrl+Q` |

## Instalação

O jeito mais simples no Debian, Ubuntu, Fedora e openSUSE é a
[página de download](https://software.opensuse.org/download.html?project=home%3Anachopro&package=trigonum): você escolhe sua distro e ela mostra como adicionar o repositório,
ou oferece a instalação em um clique. Abaixo estão os mesmos passos para cada distro.

### Arch Linux e derivadas

Está no AUR como [`trigonum`](https://aur.archlinux.org/packages/trigonum):

```sh
yay -S trigonum        # ou: paru -S trigonum
```

Sem um assistente de AUR:

```sh
git clone https://aur.archlinux.org/trigonum.git
cd trigonum
makepkg -si
```

### Debian 13 (trixie)

```sh
sudo install -d /etc/apt/keyrings
curl -fsSL https://download.opensuse.org/repositories/home:/nachopro/Debian_13/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/trigonum.gpg
echo 'deb [signed-by=/etc/apt/keyrings/trigonum.gpg] https://download.opensuse.org/repositories/home:/nachopro/Debian_13/ /' | sudo tee /etc/apt/sources.list.d/trigonum.list
sudo apt update
sudo apt install trigonum
```

### Ubuntu 26.04

```sh
sudo install -d /etc/apt/keyrings
curl -fsSL https://download.opensuse.org/repositories/home:/nachopro/xUbuntu_26.04/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/trigonum.gpg
echo 'deb [signed-by=/etc/apt/keyrings/trigonum.gpg] https://download.opensuse.org/repositories/home:/nachopro/xUbuntu_26.04/ /' | sudo tee /etc/apt/sources.list.d/trigonum.list
sudo apt update
sudo apt install trigonum
```

### Fedora 44 e 45

```sh
sudo dnf config-manager addrepo --from-repofile=https://download.opensuse.org/repositories/home:/nachopro/Fedora_$(rpm -E %fedora)/home:nachopro.repo
sudo dnf install trigonum
```

### openSUSE Tumbleweed

```sh
sudo zypper addrepo https://download.opensuse.org/repositories/home:/nachopro/openSUSE_Tumbleweed/home:nachopro.repo
sudo zypper refresh
sudo zypper install trigonum
```

O `zypper refresh` pede para confirmar a chave do repositório: responda `a` (confiar sempre).

Com o repositório adicionado, as versões novas chegam junto com as atualizações normais do sistema.
Os pacotes são compilados no Open Build Service, no projeto `home:nachopro`.

## Compilação

Requisitos: Rust ≥ 1.85 (cargo), Meson ≥ 1.1, GTK ≥ 4.12 e gettext.

```sh
meson setup build --buildtype=release
meson compile -C build
meson test -C build
sudo meson install -C build
```

Para desenvolvimento, basta `cargo run -- modelo.stl` (a interface fica em inglês, a
menos que as traduções estejam instaladas). Para testar as traduções sem instalar:

```sh
meson compile -C build
TRIGONUM_LOCALEDIR=build/po LANGUAGE=pt build/trigonum modelo.stl
```

## Pacotes e CI

Os arquivos de empacotamento ficam em `debian/`, `packaging/rpm/trigonum.spec` e
`packaging/arch/PKGBUILD`. O tarball de cada versão inclui os crates em `vendor/`,
para que Debian, Ubuntu, Fedora e openSUSE possam compilar sem rede.

O CI completo (`.forgejo/workflows/ci.yml`) compila, testa e instala o pacote real no
Debian 13, Ubuntu 26.04, Arch, Fedora e openSUSE Tumbleweed. Ele roda no PR de cada RC e sob
demanda (veja [CONTRIBUTING.pt.md](CONTRIBUTING.pt.md)). Para rodar o mesmo localmente com Docker:

```sh
build-aux/ci/run-local.sh all        # ou: debian, ubuntu, arch, fedora, opensuse
```

Os pacotes ficam em `ci-out/`. A versão mínima do Rust é a 1.85, a do Debian 13;
o clippy avisa se for usado algo mais novo.

O CI usa imagens próprias (`trigonum-ci/debian`, `ubuntu`, `arch`, `fedora` e `opensuse`)
com tudo pré-instalado: ferramentas, dependências, os crates do `Cargo.lock` e, na do
Arch, as dependências do clippy já compiladas. Elas são construídas no servidor do runner com
`build-aux/ci/build-images.sh`, e `build-aux/ci/host/` traz um timer do systemd que as
reconstrói toda semana e limpa o que os jobs deixam para trás. Se você mudar as dependências,
o CI baixa o que for novo até a próxima reconstrução; para não esperar, rode `build-images.sh` à mão.

## Traduções

Os textos são extraídos com gettext (pasta `po/`). Para adicionar um idioma:

1. Adicione o código do idioma em `po/LINGUAS`.
2. Rode `meson compile -C build trigonum-pot` para atualizar o modelo.
3. Rode `msginit -i po/trigonum.pot -l <código> -o po/<código>.po`, traduza e envie um pull request.

Depois de mudar textos no código: `meson compile -C build trigonum-update-po`.

## Contribuir e publicar versões

As features vão por PR para um branch temporário `next`, que é renomeado para
`release/X.Y.Z` quando uma versão sai. O PR dele para `main` é o RC: roda o CI completo nas
5 distros e, ao ser mesclado, a versão é publicada. Os detalhes estão em
[CONTRIBUTING.pt.md](CONTRIBUTING.pt.md).

## Licença

GPL-3.0-or-later. Veja [LICENSE](LICENSE).
