"""List the imports of a PEF container (classic Mac OS PowerPC code fragment).

Usage: python3 -I tools/pef_imports.py DATA_FORK > imports.tsv
Prints one "library<TAB>symbol" line per import.
"""
import struct, sys

def imports(path):
    d = open(path, 'rb').read()
    assert d[:8] == b'Joy!peff', 'not a PEF container'
    nsec = struct.unpack('>H', d[32:34])[0]
    for i in range(nsec):
        off = 40 + 28 * i
        (name_off, def_addr, exec_size, init_size, raw_size, cont_off,
         kind, share, align, _) = struct.unpack('>iIIIIIBBBB', d[off:off + 28])
        if kind != 4:  # loader section
            continue
        L = cont_off
        (main_sec, main_off, init_sec, init_off, term_sec, term_off,
         n_libs, n_syms, n_reloc_secs, reloc_instr_off, strtab_off,
         hash_off, hash_power, n_exports) = struct.unpack(
            '>iIiIiIIIIIIIII', d[L:L + 56])
        strtab = L + strtab_off
        def cstr(o):
            e = d.index(b'\0', strtab + o)
            return d[strtab + o:e].decode('mac_roman')
        libs = []
        p = L + 56
        for _ in range(n_libs):
            (lname, oldv, curv, nsyms, first, opts, _r1, _r2) = struct.unpack(
                '>IIIIIBBH', d[p:p + 24])
            libs.append((cstr(lname), first, nsyms))
            p += 24
        syms = []
        for j in range(n_syms):
            w = struct.unpack('>I', d[p + 4 * j:p + 4 * j + 4])[0]
            syms.append(cstr(w & 0xFFFFFF))
        for lname, first, n in libs:
            for s in syms[first:first + n]:
                print(f'{lname}\t{s}')

imports(sys.argv[1])
