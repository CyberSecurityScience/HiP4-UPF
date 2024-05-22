
import re
import sys
import os


template_file = "settings_template.p4"
target_file = "settings.p4"

def try_value(value_simple: int, value_complex: int, template: str, write_only = False) -> bool :
    print('===================================================Trying', value_simple, value_complex)
    template = template.replace("<%VALUE_SIMPLE%>", f"{value_simple}").replace("<%VALUE_COMPLEX%>", f"{value_complex}")
    with open(target_file, 'w', encoding = 'utf-8') as fp :
        fp.write(template)
    if write_only :
        return True
    retval = os.system("bf-p4c upf.p4 --create-graphs --display-power-budget --log-hashes -g")
    return retval == 0

def main2() :
    with open(template_file, 'r', encoding = 'utf-8') as fp :
        template = fp.read()

    val = 1
    lb = 0
    ub = 0
    while True :
        if try_value(val, template) :
            val *= 2
        else :
            lb = val // 2
            ub = val
            break

    while lb < ub :
        val = (lb + ub) // 2
        if val == lb :
            break
        if try_value(val, template) :
            lb = val
        else :
            ub = val - 1
    
    while True :
        if try_value(val, template) :
            break
        else :
            val -= 1

    try_value(val, template, write_only = True)

def main() :
    with open(template_file, 'r', encoding = 'utf-8') as fp :
        template = fp.read()
    with open('gs-result.txt', 'w') as fp :
        for ratio in [5, 10, 20, 50] :
            fp.write(f'===Complex ratio {ratio}===')
            for val in range(30, 200) :
                n_simple = val * 1024
                n_complex = int(round(n_simple / ratio))
                if try_value(n_simple, n_complex, template) :
                    fp.write(f'n_complex:{n_complex} n_simple:{n_simple} ok\n')
                else :
                    fp.write(f'n_complex:{n_complex} n_simple:{n_simple} failed\n')

if __name__ == '__main__' :
    main()
