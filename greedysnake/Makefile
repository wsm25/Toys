objects = main.o apple.o snake.o wSDL.o ico.o
SDLlibs = -lmingw32 -lSDL2main -lSDL2 
libs	= -lsetupapi -lwinmm -limm32 -lversion -lole32 -loleaut32 -lgdi32
dynlibs = -Wl,-Bstatic -lgcc -lstdc++ -lwinpthread
CPPFLAGS= -O3
CXX=x86_64-w64-mingw32-g++
greedysnake.exe: $(objects)
	$(CXX) -static -O3 -o $@ $(objects) $(SDLlibs) $(libs)
	x86_64-w64-mingw32-strip $@

$(objects): consts.h apple.h snake.h wSDL.h

ico.o: resource.rc ICON.ico
	windres -i "resource.rc" -o "ico.o"

dynamic: $(objects)
	$(CXX) -O3 -o greedysnake.exe $(objects) $(SDLlibs) $(dynlibs)
	strip greedysnake.exe

dyn: $(objects)
	$(CXX) -O3 -o greedysnake.exe $(objects) $(SDLlibs)
	strip greedysnake.exe
	
# .PHONY : clean
clean :
	-del edit $(objects)

haha :
	echo $(RM)
