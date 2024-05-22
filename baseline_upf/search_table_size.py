
import re
import sys
import os


template_file = "settings_template.p4"
target_file = "settings.p4"

def try_value(value: int, template: str, write_only = False) -> bool :
    print('===================================================Trying', value)
    template = template.replace("<%VALUE_SIMPLE%>", f"1024 * {value}")
    value_complex = 0
    template = template.replace("<%VALUE_COMPLEX%>", f"1024 * {value_complex}")
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
    with open('gs-result-simple-only.txt', 'w') as fp :
        for val in range(70, 111) :
            if try_value(val, template) :
                fp.write(f'{val} ok\n')
            else :
                fp.write(f'{val} failed\n')

if __name__ == '__main__' :
    main()
