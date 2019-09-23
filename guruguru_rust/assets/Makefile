.PHONY: all clean

all: images

clean:
	-@rm *.raw

images: $(patsubst %.png,%.raw,$(wildcard *.png))

%.raw: %.png
	convert $< -depth 1 -background white -flatten gray:$@