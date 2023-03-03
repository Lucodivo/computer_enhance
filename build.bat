@echo off

set CommonCompilerFlags=-MTd -nologo -fp:fast -Gm- -GR- -EHa- -Od -Oi -WX -W4 -wd4201 -wd4100 -wd4189 -wd4505 -wd4530 -DHANDMADE_INTERNAL=1 -DHANDMADE_SLOW=1 -DHANDMADE_WIN32=1 -D_CRT_SECURE_NO_WARNINGS=1 -FC -Z7
set CommonLinkerFlags= -incremental:no -opt:ref user32.lib gdi32.lib winmm.lib

IF NOT EXIST build mkdir build
pushd build

del *.pdb > NUL 2> NUL
REM Optimization switches /O2
cl %CommonCompilerFlags% ..\src\main.cpp -Fmmain.map /link -incremental:no -opt:ref -PDB:main.pdb
popd
