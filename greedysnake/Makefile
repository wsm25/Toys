objects = main.o apple.o snake.o wSDL.o ico.o
libs	= -lmingw32 -lSDL2main -lSDL2 -lsetupapi -lwinmm -limm32 -lversion -lole32 -loleaut32 -lgdi32
CPPFLAGS= -O3
greedysnake.exe: $(objects)
	g++ -static -O3 -o greedysnake.exe $(objects) $(libs)
	strip greedysnake.exe

$(objects): consts.h apple.h snake.h wSDL.h

ico.o: resource.rc ICON.ico
	windres -i "resource.rc" -o "ico.o"

# .PHONY : clean
clean :
	del edit $(objects)