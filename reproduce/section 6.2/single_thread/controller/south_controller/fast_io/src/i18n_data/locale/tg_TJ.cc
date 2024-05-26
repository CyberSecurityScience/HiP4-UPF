﻿#include"../localedef.h"

namespace fast_io_i18n
{
namespace
{

inline constexpr std::size_t numeric_grouping_storage[]{3};

inline constexpr lc_all lc_all_global{.identification={.name=tsc("tg_TJ"),.encoding=tsc(FAST_IO_LOCALE_ENCODING),.title=tsc("Tajik language locale for Tajikistan"),.source=tsc("Roger Kovacs\t\t;\t\tfast_io"),.address=tsc("https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc("Pablo Saratxaga, Roger Kovacs\t\t;\t\tfast_io"),.email=tsc("pablo@mandrakesoft.com, ROGERKO@micromotion.com;euloanty@live.com"),.tel=tsc(""),.fax=tsc(""),.language=tsc("Tajik"),.territory=tsc("Tajikistan"),.revision=tsc("0.4"),.date=tsc("2004-01-09")},.monetary={.int_curr_symbol=tsc("TJS "),.currency_symbol=tsc("руб"),.mon_decimal_point=tsc("."),.mon_thousands_sep=tsc(" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(""),.negative_sign=tsc("-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(","),.thousands_sep=tsc("."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc("Вск"),tsc("Пнд"),tsc("Втр"),tsc("Срд"),tsc("Чтв"),tsc("Птн"),tsc("Сбт")},.day={tsc("Воскресенье"),tsc("Понедельник"),tsc("Вторник"),tsc("Среда"),tsc("Четверг"),tsc("Пятница"),tsc("Суббота")},.abmon={tsc("Янв"),tsc("Фев"),tsc("Мар"),tsc("Апр"),tsc("Май"),tsc("Июн"),tsc("Июл"),tsc("Авг"),tsc("Сен"),tsc("Окт"),tsc("Ноя"),tsc("Дек")},.mon={tsc("Января"),tsc("Февраля"),tsc("Марта"),tsc("Апреля"),tsc("Мая"),tsc("Июня"),tsc("Июля"),tsc("Августа"),tsc("Сентября"),tsc("Октября"),tsc("Ноября"),tsc("Декабря")},.d_t_fmt=tsc("%a %d %b %Y %T"),.d_fmt=tsc("%d.%m.%Y"),.t_fmt=tsc("%T"),.t_fmt_ampm=tsc(""),.date_fmt=tsc("%a %d %b %Y %T %Z"),.am_pm={tsc(""),tsc("")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc("^[+1yYҲҳХхДд]"),.noexpr=tsc("^[-0nNНн]"),.yesstr=tsc("ҳа"),.nostr=tsc("не")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc("+%c %a%t%l"),.int_select=tsc("00"),.int_prefix=tsc("992")},.name={.name_fmt=tsc("%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc("%a%N%f%N%d%N%b%N%h %s %e %r%N%T %z%N%c%N"),.country_name=tsc("Тоҷикистон"),.country_ab2=tsc("TJ"),.country_ab3=tsc("TJK"),.country_num=762,.country_car=tsc("TJ"),.lang_name=tsc("тоҷикӣ"),.lang_ab=tsc("tg"),.lang_term=tsc("tgk"),.lang_lib=tsc("tgk")},.measurement={.measurement=1}};

inline constexpr wlc_all wlc_all_global{.identification={.name=tsc(L"tg_TJ"),.encoding=tsc(FAST_IO_LOCALE_LENCODING),.title=tsc(L"Tajik language locale for Tajikistan"),.source=tsc(L"Roger Kovacs\t\t;\t\tfast_io"),.address=tsc(L"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(L"Pablo Saratxaga, Roger Kovacs\t\t;\t\tfast_io"),.email=tsc(L"pablo@mandrakesoft.com, ROGERKO@micromotion.com;euloanty@live.com"),.tel=tsc(L""),.fax=tsc(L""),.language=tsc(L"Tajik"),.territory=tsc(L"Tajikistan"),.revision=tsc(L"0.4"),.date=tsc(L"2004-01-09")},.monetary={.int_curr_symbol=tsc(L"TJS "),.currency_symbol=tsc(L"руб"),.mon_decimal_point=tsc(L"."),.mon_thousands_sep=tsc(L" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(L""),.negative_sign=tsc(L"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(L","),.thousands_sep=tsc(L"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(L"Вск"),tsc(L"Пнд"),tsc(L"Втр"),tsc(L"Срд"),tsc(L"Чтв"),tsc(L"Птн"),tsc(L"Сбт")},.day={tsc(L"Воскресенье"),tsc(L"Понедельник"),tsc(L"Вторник"),tsc(L"Среда"),tsc(L"Четверг"),tsc(L"Пятница"),tsc(L"Суббота")},.abmon={tsc(L"Янв"),tsc(L"Фев"),tsc(L"Мар"),tsc(L"Апр"),tsc(L"Май"),tsc(L"Июн"),tsc(L"Июл"),tsc(L"Авг"),tsc(L"Сен"),tsc(L"Окт"),tsc(L"Ноя"),tsc(L"Дек")},.mon={tsc(L"Января"),tsc(L"Февраля"),tsc(L"Марта"),tsc(L"Апреля"),tsc(L"Мая"),tsc(L"Июня"),tsc(L"Июля"),tsc(L"Августа"),tsc(L"Сентября"),tsc(L"Октября"),tsc(L"Ноября"),tsc(L"Декабря")},.d_t_fmt=tsc(L"%a %d %b %Y %T"),.d_fmt=tsc(L"%d.%m.%Y"),.t_fmt=tsc(L"%T"),.t_fmt_ampm=tsc(L""),.date_fmt=tsc(L"%a %d %b %Y %T %Z"),.am_pm={tsc(L""),tsc(L"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(L"^[+1yYҲҳХхДд]"),.noexpr=tsc(L"^[-0nNНн]"),.yesstr=tsc(L"ҳа"),.nostr=tsc(L"не")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(L"+%c %a%t%l"),.int_select=tsc(L"00"),.int_prefix=tsc(L"992")},.name={.name_fmt=tsc(L"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(L"%a%N%f%N%d%N%b%N%h %s %e %r%N%T %z%N%c%N"),.country_name=tsc(L"Тоҷикистон"),.country_ab2=tsc(L"TJ"),.country_ab3=tsc(L"TJK"),.country_num=762,.country_car=tsc(L"TJ"),.lang_name=tsc(L"тоҷикӣ"),.lang_ab=tsc(L"tg"),.lang_term=tsc(L"tgk"),.lang_lib=tsc(L"tgk")},.measurement={.measurement=1}};

inline constexpr u8lc_all u8lc_all_global{.identification={.name=tsc(u8"tg_TJ"),.encoding=tsc(FAST_IO_LOCALE_u8ENCODING),.title=tsc(u8"Tajik language locale for Tajikistan"),.source=tsc(u8"Roger Kovacs\t\t;\t\tfast_io"),.address=tsc(u8"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u8"Pablo Saratxaga, Roger Kovacs\t\t;\t\tfast_io"),.email=tsc(u8"pablo@mandrakesoft.com, ROGERKO@micromotion.com;euloanty@live.com"),.tel=tsc(u8""),.fax=tsc(u8""),.language=tsc(u8"Tajik"),.territory=tsc(u8"Tajikistan"),.revision=tsc(u8"0.4"),.date=tsc(u8"2004-01-09")},.monetary={.int_curr_symbol=tsc(u8"TJS "),.currency_symbol=tsc(u8"руб"),.mon_decimal_point=tsc(u8"."),.mon_thousands_sep=tsc(u8" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(u8""),.negative_sign=tsc(u8"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u8","),.thousands_sep=tsc(u8"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u8"Вск"),tsc(u8"Пнд"),tsc(u8"Втр"),tsc(u8"Срд"),tsc(u8"Чтв"),tsc(u8"Птн"),tsc(u8"Сбт")},.day={tsc(u8"Воскресенье"),tsc(u8"Понедельник"),tsc(u8"Вторник"),tsc(u8"Среда"),tsc(u8"Четверг"),tsc(u8"Пятница"),tsc(u8"Суббота")},.abmon={tsc(u8"Янв"),tsc(u8"Фев"),tsc(u8"Мар"),tsc(u8"Апр"),tsc(u8"Май"),tsc(u8"Июн"),tsc(u8"Июл"),tsc(u8"Авг"),tsc(u8"Сен"),tsc(u8"Окт"),tsc(u8"Ноя"),tsc(u8"Дек")},.mon={tsc(u8"Января"),tsc(u8"Февраля"),tsc(u8"Марта"),tsc(u8"Апреля"),tsc(u8"Мая"),tsc(u8"Июня"),tsc(u8"Июля"),tsc(u8"Августа"),tsc(u8"Сентября"),tsc(u8"Октября"),tsc(u8"Ноября"),tsc(u8"Декабря")},.d_t_fmt=tsc(u8"%a %d %b %Y %T"),.d_fmt=tsc(u8"%d.%m.%Y"),.t_fmt=tsc(u8"%T"),.t_fmt_ampm=tsc(u8""),.date_fmt=tsc(u8"%a %d %b %Y %T %Z"),.am_pm={tsc(u8""),tsc(u8"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(u8"^[+1yYҲҳХхДд]"),.noexpr=tsc(u8"^[-0nNНн]"),.yesstr=tsc(u8"ҳа"),.nostr=tsc(u8"не")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u8"+%c %a%t%l"),.int_select=tsc(u8"00"),.int_prefix=tsc(u8"992")},.name={.name_fmt=tsc(u8"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(u8"%a%N%f%N%d%N%b%N%h %s %e %r%N%T %z%N%c%N"),.country_name=tsc(u8"Тоҷикистон"),.country_ab2=tsc(u8"TJ"),.country_ab3=tsc(u8"TJK"),.country_num=762,.country_car=tsc(u8"TJ"),.lang_name=tsc(u8"тоҷикӣ"),.lang_ab=tsc(u8"tg"),.lang_term=tsc(u8"tgk"),.lang_lib=tsc(u8"tgk")},.measurement={.measurement=1}};

inline constexpr u16lc_all u16lc_all_global{.identification={.name=tsc(u"tg_TJ"),.encoding=tsc(FAST_IO_LOCALE_uENCODING),.title=tsc(u"Tajik language locale for Tajikistan"),.source=tsc(u"Roger Kovacs\t\t;\t\tfast_io"),.address=tsc(u"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u"Pablo Saratxaga, Roger Kovacs\t\t;\t\tfast_io"),.email=tsc(u"pablo@mandrakesoft.com, ROGERKO@micromotion.com;euloanty@live.com"),.tel=tsc(u""),.fax=tsc(u""),.language=tsc(u"Tajik"),.territory=tsc(u"Tajikistan"),.revision=tsc(u"0.4"),.date=tsc(u"2004-01-09")},.monetary={.int_curr_symbol=tsc(u"TJS "),.currency_symbol=tsc(u"руб"),.mon_decimal_point=tsc(u"."),.mon_thousands_sep=tsc(u" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(u""),.negative_sign=tsc(u"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u","),.thousands_sep=tsc(u"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u"Вск"),tsc(u"Пнд"),tsc(u"Втр"),tsc(u"Срд"),tsc(u"Чтв"),tsc(u"Птн"),tsc(u"Сбт")},.day={tsc(u"Воскресенье"),tsc(u"Понедельник"),tsc(u"Вторник"),tsc(u"Среда"),tsc(u"Четверг"),tsc(u"Пятница"),tsc(u"Суббота")},.abmon={tsc(u"Янв"),tsc(u"Фев"),tsc(u"Мар"),tsc(u"Апр"),tsc(u"Май"),tsc(u"Июн"),tsc(u"Июл"),tsc(u"Авг"),tsc(u"Сен"),tsc(u"Окт"),tsc(u"Ноя"),tsc(u"Дек")},.mon={tsc(u"Января"),tsc(u"Февраля"),tsc(u"Марта"),tsc(u"Апреля"),tsc(u"Мая"),tsc(u"Июня"),tsc(u"Июля"),tsc(u"Августа"),tsc(u"Сентября"),tsc(u"Октября"),tsc(u"Ноября"),tsc(u"Декабря")},.d_t_fmt=tsc(u"%a %d %b %Y %T"),.d_fmt=tsc(u"%d.%m.%Y"),.t_fmt=tsc(u"%T"),.t_fmt_ampm=tsc(u""),.date_fmt=tsc(u"%a %d %b %Y %T %Z"),.am_pm={tsc(u""),tsc(u"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(u"^[+1yYҲҳХхДд]"),.noexpr=tsc(u"^[-0nNНн]"),.yesstr=tsc(u"ҳа"),.nostr=tsc(u"не")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u"+%c %a%t%l"),.int_select=tsc(u"00"),.int_prefix=tsc(u"992")},.name={.name_fmt=tsc(u"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(u"%a%N%f%N%d%N%b%N%h %s %e %r%N%T %z%N%c%N"),.country_name=tsc(u"Тоҷикистон"),.country_ab2=tsc(u"TJ"),.country_ab3=tsc(u"TJK"),.country_num=762,.country_car=tsc(u"TJ"),.lang_name=tsc(u"тоҷикӣ"),.lang_ab=tsc(u"tg"),.lang_term=tsc(u"tgk"),.lang_lib=tsc(u"tgk")},.measurement={.measurement=1}};

inline constexpr u32lc_all u32lc_all_global{.identification={.name=tsc(U"tg_TJ"),.encoding=tsc(FAST_IO_LOCALE_UENCODING),.title=tsc(U"Tajik language locale for Tajikistan"),.source=tsc(U"Roger Kovacs\t\t;\t\tfast_io"),.address=tsc(U"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(U"Pablo Saratxaga, Roger Kovacs\t\t;\t\tfast_io"),.email=tsc(U"pablo@mandrakesoft.com, ROGERKO@micromotion.com;euloanty@live.com"),.tel=tsc(U""),.fax=tsc(U""),.language=tsc(U"Tajik"),.territory=tsc(U"Tajikistan"),.revision=tsc(U"0.4"),.date=tsc(U"2004-01-09")},.monetary={.int_curr_symbol=tsc(U"TJS "),.currency_symbol=tsc(U"руб"),.mon_decimal_point=tsc(U"."),.mon_thousands_sep=tsc(U" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(U""),.negative_sign=tsc(U"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(U","),.thousands_sep=tsc(U"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(U"Вск"),tsc(U"Пнд"),tsc(U"Втр"),tsc(U"Срд"),tsc(U"Чтв"),tsc(U"Птн"),tsc(U"Сбт")},.day={tsc(U"Воскресенье"),tsc(U"Понедельник"),tsc(U"Вторник"),tsc(U"Среда"),tsc(U"Четверг"),tsc(U"Пятница"),tsc(U"Суббота")},.abmon={tsc(U"Янв"),tsc(U"Фев"),tsc(U"Мар"),tsc(U"Апр"),tsc(U"Май"),tsc(U"Июн"),tsc(U"Июл"),tsc(U"Авг"),tsc(U"Сен"),tsc(U"Окт"),tsc(U"Ноя"),tsc(U"Дек")},.mon={tsc(U"Января"),tsc(U"Февраля"),tsc(U"Марта"),tsc(U"Апреля"),tsc(U"Мая"),tsc(U"Июня"),tsc(U"Июля"),tsc(U"Августа"),tsc(U"Сентября"),tsc(U"Октября"),tsc(U"Ноября"),tsc(U"Декабря")},.d_t_fmt=tsc(U"%a %d %b %Y %T"),.d_fmt=tsc(U"%d.%m.%Y"),.t_fmt=tsc(U"%T"),.t_fmt_ampm=tsc(U""),.date_fmt=tsc(U"%a %d %b %Y %T %Z"),.am_pm={tsc(U""),tsc(U"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(U"^[+1yYҲҳХхДд]"),.noexpr=tsc(U"^[-0nNНн]"),.yesstr=tsc(U"ҳа"),.nostr=tsc(U"не")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(U"+%c %a%t%l"),.int_select=tsc(U"00"),.int_prefix=tsc(U"992")},.name={.name_fmt=tsc(U"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(U"%a%N%f%N%d%N%b%N%h %s %e %r%N%T %z%N%c%N"),.country_name=tsc(U"Тоҷикистон"),.country_ab2=tsc(U"TJ"),.country_ab3=tsc(U"TJK"),.country_num=762,.country_car=tsc(U"TJ"),.lang_name=tsc(U"тоҷикӣ"),.lang_ab=tsc(U"tg"),.lang_term=tsc(U"tgk"),.lang_lib=tsc(U"tgk")},.measurement={.measurement=1}};


}
}

#include"../main.h"