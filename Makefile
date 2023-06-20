
RUBY_GENERATION_FILES = vtparse_gen_c_tables.rb vtparse_tables.rb

all: vtparse_enums.h vtparse_names.c vtparse_table.c vtparse_table.h vtparse libvtparse.a

clean:
	rm -f vtparse_enums.h vtparse_names.c vtparse_table.c vtparse_table.h vtparse *.o libvtparse.a

vtparse_enums.h: $(RUBY_GENERATION_FILES)
	ruby vtparse_gen_c_tables.rb

vtparse_names.c: $(RUBY_GENERATION_FILES)
	ruby vtparse_gen_c_tables.rb

vtparse_table.c: $(RUBY_GENERATION_FILES)
	ruby vtparse_gen_c_tables.rb

vtparse_table.h: $(RUBY_GENERATION_FILES)
	ruby vtparse_gen_c_tables.rb

vtparse: vtparse.c vtparse.h vtparse_enums.h vtparse_names.c vtparse_table.c vtparse_table.h vtparse_test.c
	gcc -Wall -o $@ vtparse_test.c vtparse.c vtparse_table.c

libvtparse.a: vtparse.o vtparse_table.o
	rm -f $@
	ar r $@ $^
	ranlib $@

.c.o:
	gcc -O3 -Wall -o $@ -c $<


.PHONY: all clean

