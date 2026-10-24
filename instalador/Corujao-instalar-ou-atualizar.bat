@echo off
rem Corujão — instala ou atualiza no Windows, sem senha de administrador.
rem Clique duas vezes. Baixa a última versão do GitHub para %LOCALAPPDATA%\corujao e cria um
rem atalho na Área de Trabalho. Se já houver Corujão, só atualiza quando existir versão nova.
rem Guarda a versão instalada em versao.txt ao lado do corujao.exe.
rem
rem Hipóteses (não há Windows nesta máquina para testar): Windows 10 (2018+) ou 11, que já
rem trazem curl, tar e PowerShell; console em UTF-8 (chcp 65001) para os acentos.
chcp 65001 >nul
setlocal EnableExtensions
title Corujão - instalar ou atualizar

set "REPO=nicho55/corujao"
set "PACOTE=corujao-windows-x64.zip"
set "DESTINO=%LOCALAPPDATA%\corujao"
set "ARQ_VERSAO=%DESTINO%\versao.txt"
set "TRABALHO=%TEMP%\corujao-instalador"
set "VERSAO_ATUAL="
set "VERSAO_NOVA="

echo.
echo  ==========================================================
echo    Corujão
echo    Instalar ou atualizar no Windows
echo  ==========================================================
echo.
echo  Este programa baixa a última versão do Corujão do GitHub e deixa
echo  tudo pronto em %DESTINO%,
echo  com um atalho na Área de Trabalho. Não pede senha de administrador.
echo.

echo  [1/5] Conferindo o que o seu Windows já tem...
where curl >nul 2>&1
if errorlevel 1 goto :sem_curl
where tar >nul 2>&1
if errorlevel 1 goto :sem_tar
echo         Tudo certo: curl e tar encontrados.

echo  [2/5] Perguntando ao GitHub qual é a última versão...
set "URL_FINAL="
for /f "usebackq delims=" %%u in (`curl -s -L --max-time 25 -o NUL -w "%%{url_effective}" "https://github.com/%REPO%/releases/latest"`) do set "URL_FINAL=%%u"
if not defined URL_FINAL goto :sem_internet
set "VERSAO_NOVA=%URL_FINAL:*/releases/tag/=%"
if "%VERSAO_NOVA%"=="%URL_FINAL%" goto :sem_release
echo         Última versão publicada: %VERSAO_NOVA%

if exist "%ARQ_VERSAO%" set /p VERSAO_ATUAL=<"%ARQ_VERSAO%"
if not defined VERSAO_ATUAL goto :instalar_novo
if "%VERSAO_ATUAL%"=="%VERSAO_NOVA%" goto :ja_atualizado
echo  [3/5] Você tem a %VERSAO_ATUAL%; vou atualizar para a %VERSAO_NOVA%.
goto :baixar
:instalar_novo
echo  [3/5] O Corujão ainda não está neste computador; vou instalar a %VERSAO_NOVA%.

:baixar
echo  [4/5] Baixando o pacote (%PACOTE%)...
if exist "%TRABALHO%" rd /s /q "%TRABALHO%" >nul 2>&1
md "%TRABALHO%" >nul 2>&1
curl -# -f -L --connect-timeout 25 -o "%TRABALHO%\%PACOTE%" "https://github.com/%REPO%/releases/download/%VERSAO_NOVA%/%PACOTE%"
if errorlevel 1 goto :falha_download
if not exist "%TRABALHO%\%PACOTE%" goto :falha_download
echo         Pacote baixado.

echo  [5/5] Instalando em %DESTINO%...
tasklist /FI "IMAGENAME eq corujao.exe" 2>nul | find /I "corujao.exe" >nul
if not errorlevel 1 (
  echo         O Corujão está aberto; vou fechá-lo para atualizar.
  taskkill /IM corujao.exe /F >nul 2>&1
  timeout /t 2 /nobreak >nul
)
tar -xf "%TRABALHO%\%PACOTE%" -C "%TRABALHO%"
if errorlevel 1 goto :falha_extrair
if not exist "%TRABALHO%\corujao\corujao.exe" goto :falha_extrair
if not exist "%DESTINO%" md "%DESTINO%" >nul 2>&1
robocopy "%TRABALHO%\corujao" "%DESTINO%" /E /NFL /NDL /NJH /NJS /NP >nul
if errorlevel 8 goto :falha_copiar
if not exist "%DESTINO%\corujao.exe" goto :sumiu_antivirus
>"%ARQ_VERSAO%" echo %VERSAO_NOVA%
rd /s /q "%TRABALHO%" >nul 2>&1

set "AREA="
for /f "usebackq delims=" %%d in (`powershell -NoProfile -Command "[Environment]::GetFolderPath('Desktop')"`) do set "AREA=%%d"
if not defined AREA set "AREA=%USERPROFILE%\Desktop"
set "ATALHO=%AREA%\Corujão.lnk"
powershell -NoProfile -ExecutionPolicy Bypass -Command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%ATALHO%');$s.TargetPath='%DESTINO%\corujao.exe';$s.WorkingDirectory='%DESTINO%';$s.IconLocation='%DESTINO%\corujao.exe';$s.Description='Corujão';$s.Save()" >nul 2>&1
if exist "%ATALHO%" (
  echo         Atalho "Corujão" criado na Área de Trabalho.
) else (
  echo         Não consegui criar o atalho. O programa está em %DESTINO%\corujao.exe
)

echo.
echo  ==========================================================
if defined VERSAO_ATUAL (
  echo    Pronto. Corujão atualizado da %VERSAO_ATUAL% para a %VERSAO_NOVA%.
) else (
  echo    Pronto. Corujão %VERSAO_NOVA% instalado.
)
echo  ==========================================================
echo.
echo  Na primeira vez que abrir, o Windows pode mostrar uma tela azul
echo  dizendo que "protegeu o seu PC". É porque o programa não tem um
echo  certificado pago; não é vírus. Clique em "Mais informações" e
echo  depois em "Executar assim mesmo". Só acontece uma vez.
echo.
echo  O Corujão não vem com jogo nenhum: ele é o launcher. Para jogar, você
echo  compila a recomp a partir do seu próprio disco; o LEIAME explica.
echo.
choice /C SN /N /M "  Abrir o Corujão agora? [S/N] "
if errorlevel 2 goto :fim
start "" "%DESTINO%\corujao.exe"
goto :fim

:ja_atualizado
echo  [3/5] Você já tem a última versão (%VERSAO_ATUAL%). Nada a fazer.
echo.
echo  O Corujão está em %DESTINO%\corujao.exe
goto :fim

:sem_curl
echo.
echo  Este Windows não tem o programa "curl", que vem de fábrica no
echo  Windows 10 (desde 2018) e no Windows 11.
echo  O que fazer: atualize o Windows pelo Windows Update e rode este
echo  arquivo de novo. Ou baixe à mão em
echo  https://github.com/%REPO%/releases/latest
echo  (arquivo %PACOTE%), extraia a pasta e abra o corujao.exe.
goto :fim_com_erro

:sem_tar
echo.
echo  Este Windows não tem o programa "tar", que vem de fábrica no
echo  Windows 10 (desde 2018) e no Windows 11.
echo  O que fazer: atualize o Windows pelo Windows Update e rode este
echo  arquivo de novo. Ou baixe à mão em
echo  https://github.com/%REPO%/releases/latest
echo  (arquivo %PACOTE%), extraia a pasta e abra o corujao.exe.
goto :fim_com_erro

:sem_internet
echo.
echo  Não consegui falar com o GitHub.
echo  O que fazer: veja se a internet está funcionando (abra um site no
echo  navegador) e rode este arquivo de novo. Se a internet está boa, o
echo  GitHub pode estar fora do ar: espere alguns minutos e tente de novo.
goto :fim_com_erro

:sem_release
echo.
echo  O GitHub respondeu, mas ainda não há nenhuma versão publicada do Corujão.
echo  O que fazer: avise quem te mandou este arquivo.
goto :fim_com_erro

:falha_download
echo.
echo  O download não terminou.
echo  Causas comuns: a internet caiu no meio; o antivírus bloqueou o
echo  arquivo (olhe a quarentena dele e libere a pasta %TRABALHO%);
echo  ou o GitHub está instável. Rode este arquivo de novo daqui a pouco.
goto :fim_com_erro

:falha_extrair
echo.
echo  O pacote baixado veio incompleto, ou o antivírus apagou arquivos
echo  na hora de extrair.
echo  O que fazer: rode este arquivo de novo. Se repetir, abra o antivírus,
echo  procure a quarentena e libere a pasta %TRABALHO%.
goto :fim_com_erro

:falha_copiar
echo.
echo  Não consegui copiar os arquivos para %DESTINO%.
echo  O que fazer: feche o Corujão se ele estiver aberto e rode este arquivo
echo  de novo. Se repetir, o antivírus pode estar segurando a pasta.
goto :fim_com_erro

:sumiu_antivirus
echo.
echo  Copiei os arquivos, mas o corujao.exe sumiu de %DESTINO% logo em
echo  seguida. Isso é o antivírus apagando o programa.
echo  O que fazer: abra o antivírus, restaure o corujao.exe da quarentena e
echo  adicione a pasta %DESTINO% às exceções. Depois rode este arquivo de novo.
goto :fim_com_erro

:fim_com_erro
echo.
echo  Nada foi quebrado: o que já estava instalado continua como estava.

:fim
echo.
pause
endlocal
