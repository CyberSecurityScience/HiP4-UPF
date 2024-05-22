﻿#include"../localedef.h"

namespace fast_io_i18n
{
namespace
{

inline constexpr std::size_t numeric_grouping_storage[]{3};

inline constexpr lc_all lc_all_global{.identification={.name=tsc("ug_CN"),.encoding=tsc(FAST_IO_LOCALE_ENCODING),.title=tsc("Uyghur locale for China"),.source=tsc("fast_io"),.address=tsc("https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc("Eagle Burkut\t\t;\t\tfast_io"),.email=tsc("eagle.burkut@gmail.com;euloanty@live.com"),.tel=tsc(""),.fax=tsc(""),.language=tsc("Uyghur"),.territory=tsc("China"),.revision=tsc("2.00"),.date=tsc("2011-02-26")},.monetary={.int_curr_symbol=tsc("CNY "),.currency_symbol=tsc("￥"),.mon_decimal_point=tsc("."),.mon_thousands_sep=tsc(","),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(""),.negative_sign=tsc("-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.int_p_cs_precedes=1,.int_p_sep_by_space=0,.int_n_cs_precedes=1,.int_n_sep_by_space=0,.p_sign_posn=4,.n_sign_posn=4,.int_p_sign_posn=1,.int_n_sign_posn=1},.numeric={.decimal_point=tsc("."),.thousands_sep=tsc(","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc("ي"),tsc("د"),tsc("س"),tsc("چ"),tsc("پ"),tsc("ج"),tsc("ش")},.day={tsc("يەكشەنبە"),tsc("دۈشەنبە"),tsc("سەيشەنبە"),tsc("چارشەنبە"),tsc("پەيشەنبە"),tsc("جۈمە"),tsc("شەنبە")},.abmon={tsc("يانۋار"),tsc("فېۋرال"),tsc("مارت"),tsc("ئاپرېل"),tsc("ماي"),tsc("ئىيۇن"),tsc("ئىيۇل"),tsc("ئاۋغۇست"),tsc("سېنتەبىر"),tsc("ئۆكتەبىر"),tsc("نويابىر"),tsc("دېكابىر")},.mon={tsc("يانۋار"),tsc("فېۋرال"),tsc("مارت"),tsc("ئاپرېل"),tsc("ماي"),tsc("ئىيۇن"),tsc("ئىيۇل"),tsc("ئاۋغۇست"),tsc("سېنتەبىر"),tsc("ئۆكتەبىر"),tsc("نويابىر"),tsc("دېكابىر")},.d_t_fmt=tsc("%a، %d-%m-%Y، %T"),.d_fmt=tsc("%a، %d-%m-%Y"),.t_fmt=tsc("%T"),.date_fmt=tsc("%a، %d-%m-%Y، %T"),.am_pm={tsc(""),tsc("")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc("^[+1yYھ]"),.noexpr=tsc("^[-0nNي]"),.yesstr=tsc("ھەئە"),.nostr=tsc("ياق")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc("+%c %a %l"),.tel_dom_fmt=tsc("0%a %l"),.int_select=tsc("00"),.int_prefix=tsc("86")},.name={.name_fmt=tsc("%f%t%g%t%d"),.name_gen=tsc(""),.name_miss=tsc("小姐"),.name_mr=tsc("先生"),.name_mrs=tsc("太太"),.name_ms=tsc("女士")},.address={.postal_fmt=tsc("%c%N%T%N%s %h %e %r%N%b%N%d%N%f%N%a%N"),.country_name=tsc("جۇڭگو"),.country_ab2=tsc("CN"),.country_ab3=tsc("CHN"),.country_num=156,.country_car=tsc("CHN"),.country_isbn=tsc("7"),.lang_name=tsc("ئۇيغۇرچە"),.lang_ab=tsc("ug"),.lang_term=tsc("uig"),.lang_lib=tsc("uig")},.measurement={.measurement=1}};

inline constexpr wlc_all wlc_all_global{.identification={.name=tsc(L"ug_CN"),.encoding=tsc(FAST_IO_LOCALE_LENCODING),.title=tsc(L"Uyghur locale for China"),.source=tsc(L"fast_io"),.address=tsc(L"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(L"Eagle Burkut\t\t;\t\tfast_io"),.email=tsc(L"eagle.burkut@gmail.com;euloanty@live.com"),.tel=tsc(L""),.fax=tsc(L""),.language=tsc(L"Uyghur"),.territory=tsc(L"China"),.revision=tsc(L"2.00"),.date=tsc(L"2011-02-26")},.monetary={.int_curr_symbol=tsc(L"CNY "),.currency_symbol=tsc(L"￥"),.mon_decimal_point=tsc(L"."),.mon_thousands_sep=tsc(L","),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(L""),.negative_sign=tsc(L"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.int_p_cs_precedes=1,.int_p_sep_by_space=0,.int_n_cs_precedes=1,.int_n_sep_by_space=0,.p_sign_posn=4,.n_sign_posn=4,.int_p_sign_posn=1,.int_n_sign_posn=1},.numeric={.decimal_point=tsc(L"."),.thousands_sep=tsc(L","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(L"ي"),tsc(L"د"),tsc(L"س"),tsc(L"چ"),tsc(L"پ"),tsc(L"ج"),tsc(L"ش")},.day={tsc(L"يەكشەنبە"),tsc(L"دۈشەنبە"),tsc(L"سەيشەنبە"),tsc(L"چارشەنبە"),tsc(L"پەيشەنبە"),tsc(L"جۈمە"),tsc(L"شەنبە")},.abmon={tsc(L"يانۋار"),tsc(L"فېۋرال"),tsc(L"مارت"),tsc(L"ئاپرېل"),tsc(L"ماي"),tsc(L"ئىيۇن"),tsc(L"ئىيۇل"),tsc(L"ئاۋغۇست"),tsc(L"سېنتەبىر"),tsc(L"ئۆكتەبىر"),tsc(L"نويابىر"),tsc(L"دېكابىر")},.mon={tsc(L"يانۋار"),tsc(L"فېۋرال"),tsc(L"مارت"),tsc(L"ئاپرېل"),tsc(L"ماي"),tsc(L"ئىيۇن"),tsc(L"ئىيۇل"),tsc(L"ئاۋغۇست"),tsc(L"سېنتەبىر"),tsc(L"ئۆكتەبىر"),tsc(L"نويابىر"),tsc(L"دېكابىر")},.d_t_fmt=tsc(L"%a، %d-%m-%Y، %T"),.d_fmt=tsc(L"%a، %d-%m-%Y"),.t_fmt=tsc(L"%T"),.date_fmt=tsc(L"%a، %d-%m-%Y، %T"),.am_pm={tsc(L""),tsc(L"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(L"^[+1yYھ]"),.noexpr=tsc(L"^[-0nNي]"),.yesstr=tsc(L"ھەئە"),.nostr=tsc(L"ياق")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(L"+%c %a %l"),.tel_dom_fmt=tsc(L"0%a %l"),.int_select=tsc(L"00"),.int_prefix=tsc(L"86")},.name={.name_fmt=tsc(L"%f%t%g%t%d"),.name_gen=tsc(L""),.name_miss=tsc(L"小姐"),.name_mr=tsc(L"先生"),.name_mrs=tsc(L"太太"),.name_ms=tsc(L"女士")},.address={.postal_fmt=tsc(L"%c%N%T%N%s %h %e %r%N%b%N%d%N%f%N%a%N"),.country_name=tsc(L"جۇڭگو"),.country_ab2=tsc(L"CN"),.country_ab3=tsc(L"CHN"),.country_num=156,.country_car=tsc(L"CHN"),.country_isbn=tsc(L"7"),.lang_name=tsc(L"ئۇيغۇرچە"),.lang_ab=tsc(L"ug"),.lang_term=tsc(L"uig"),.lang_lib=tsc(L"uig")},.measurement={.measurement=1}};

inline constexpr u8lc_all u8lc_all_global{.identification={.name=tsc(u8"ug_CN"),.encoding=tsc(FAST_IO_LOCALE_u8ENCODING),.title=tsc(u8"Uyghur locale for China"),.source=tsc(u8"fast_io"),.address=tsc(u8"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u8"Eagle Burkut\t\t;\t\tfast_io"),.email=tsc(u8"eagle.burkut@gmail.com;euloanty@live.com"),.tel=tsc(u8""),.fax=tsc(u8""),.language=tsc(u8"Uyghur"),.territory=tsc(u8"China"),.revision=tsc(u8"2.00"),.date=tsc(u8"2011-02-26")},.monetary={.int_curr_symbol=tsc(u8"CNY "),.currency_symbol=tsc(u8"￥"),.mon_decimal_point=tsc(u8"."),.mon_thousands_sep=tsc(u8","),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(u8""),.negative_sign=tsc(u8"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.int_p_cs_precedes=1,.int_p_sep_by_space=0,.int_n_cs_precedes=1,.int_n_sep_by_space=0,.p_sign_posn=4,.n_sign_posn=4,.int_p_sign_posn=1,.int_n_sign_posn=1},.numeric={.decimal_point=tsc(u8"."),.thousands_sep=tsc(u8","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u8"ي"),tsc(u8"د"),tsc(u8"س"),tsc(u8"چ"),tsc(u8"پ"),tsc(u8"ج"),tsc(u8"ش")},.day={tsc(u8"يەكشەنبە"),tsc(u8"دۈشەنبە"),tsc(u8"سەيشەنبە"),tsc(u8"چارشەنبە"),tsc(u8"پەيشەنبە"),tsc(u8"جۈمە"),tsc(u8"شەنبە")},.abmon={tsc(u8"يانۋار"),tsc(u8"فېۋرال"),tsc(u8"مارت"),tsc(u8"ئاپرېل"),tsc(u8"ماي"),tsc(u8"ئىيۇن"),tsc(u8"ئىيۇل"),tsc(u8"ئاۋغۇست"),tsc(u8"سېنتەبىر"),tsc(u8"ئۆكتەبىر"),tsc(u8"نويابىر"),tsc(u8"دېكابىر")},.mon={tsc(u8"يانۋار"),tsc(u8"فېۋرال"),tsc(u8"مارت"),tsc(u8"ئاپرېل"),tsc(u8"ماي"),tsc(u8"ئىيۇن"),tsc(u8"ئىيۇل"),tsc(u8"ئاۋغۇست"),tsc(u8"سېنتەبىر"),tsc(u8"ئۆكتەبىر"),tsc(u8"نويابىر"),tsc(u8"دېكابىر")},.d_t_fmt=tsc(u8"%a، %d-%m-%Y، %T"),.d_fmt=tsc(u8"%a، %d-%m-%Y"),.t_fmt=tsc(u8"%T"),.date_fmt=tsc(u8"%a، %d-%m-%Y، %T"),.am_pm={tsc(u8""),tsc(u8"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(u8"^[+1yYھ]"),.noexpr=tsc(u8"^[-0nNي]"),.yesstr=tsc(u8"ھەئە"),.nostr=tsc(u8"ياق")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u8"+%c %a %l"),.tel_dom_fmt=tsc(u8"0%a %l"),.int_select=tsc(u8"00"),.int_prefix=tsc(u8"86")},.name={.name_fmt=tsc(u8"%f%t%g%t%d"),.name_gen=tsc(u8""),.name_miss=tsc(u8"小姐"),.name_mr=tsc(u8"先生"),.name_mrs=tsc(u8"太太"),.name_ms=tsc(u8"女士")},.address={.postal_fmt=tsc(u8"%c%N%T%N%s %h %e %r%N%b%N%d%N%f%N%a%N"),.country_name=tsc(u8"جۇڭگو"),.country_ab2=tsc(u8"CN"),.country_ab3=tsc(u8"CHN"),.country_num=156,.country_car=tsc(u8"CHN"),.country_isbn=tsc(u8"7"),.lang_name=tsc(u8"ئۇيغۇرچە"),.lang_ab=tsc(u8"ug"),.lang_term=tsc(u8"uig"),.lang_lib=tsc(u8"uig")},.measurement={.measurement=1}};

inline constexpr u16lc_all u16lc_all_global{.identification={.name=tsc(u"ug_CN"),.encoding=tsc(FAST_IO_LOCALE_uENCODING),.title=tsc(u"Uyghur locale for China"),.source=tsc(u"fast_io"),.address=tsc(u"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u"Eagle Burkut\t\t;\t\tfast_io"),.email=tsc(u"eagle.burkut@gmail.com;euloanty@live.com"),.tel=tsc(u""),.fax=tsc(u""),.language=tsc(u"Uyghur"),.territory=tsc(u"China"),.revision=tsc(u"2.00"),.date=tsc(u"2011-02-26")},.monetary={.int_curr_symbol=tsc(u"CNY "),.currency_symbol=tsc(u"￥"),.mon_decimal_point=tsc(u"."),.mon_thousands_sep=tsc(u","),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(u""),.negative_sign=tsc(u"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.int_p_cs_precedes=1,.int_p_sep_by_space=0,.int_n_cs_precedes=1,.int_n_sep_by_space=0,.p_sign_posn=4,.n_sign_posn=4,.int_p_sign_posn=1,.int_n_sign_posn=1},.numeric={.decimal_point=tsc(u"."),.thousands_sep=tsc(u","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u"ي"),tsc(u"د"),tsc(u"س"),tsc(u"چ"),tsc(u"پ"),tsc(u"ج"),tsc(u"ش")},.day={tsc(u"يەكشەنبە"),tsc(u"دۈشەنبە"),tsc(u"سەيشەنبە"),tsc(u"چارشەنبە"),tsc(u"پەيشەنبە"),tsc(u"جۈمە"),tsc(u"شەنبە")},.abmon={tsc(u"يانۋار"),tsc(u"فېۋرال"),tsc(u"مارت"),tsc(u"ئاپرېل"),tsc(u"ماي"),tsc(u"ئىيۇن"),tsc(u"ئىيۇل"),tsc(u"ئاۋغۇست"),tsc(u"سېنتەبىر"),tsc(u"ئۆكتەبىر"),tsc(u"نويابىر"),tsc(u"دېكابىر")},.mon={tsc(u"يانۋار"),tsc(u"فېۋرال"),tsc(u"مارت"),tsc(u"ئاپرېل"),tsc(u"ماي"),tsc(u"ئىيۇن"),tsc(u"ئىيۇل"),tsc(u"ئاۋغۇست"),tsc(u"سېنتەبىر"),tsc(u"ئۆكتەبىر"),tsc(u"نويابىر"),tsc(u"دېكابىر")},.d_t_fmt=tsc(u"%a، %d-%m-%Y، %T"),.d_fmt=tsc(u"%a، %d-%m-%Y"),.t_fmt=tsc(u"%T"),.date_fmt=tsc(u"%a، %d-%m-%Y، %T"),.am_pm={tsc(u""),tsc(u"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(u"^[+1yYھ]"),.noexpr=tsc(u"^[-0nNي]"),.yesstr=tsc(u"ھەئە"),.nostr=tsc(u"ياق")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u"+%c %a %l"),.tel_dom_fmt=tsc(u"0%a %l"),.int_select=tsc(u"00"),.int_prefix=tsc(u"86")},.name={.name_fmt=tsc(u"%f%t%g%t%d"),.name_gen=tsc(u""),.name_miss=tsc(u"小姐"),.name_mr=tsc(u"先生"),.name_mrs=tsc(u"太太"),.name_ms=tsc(u"女士")},.address={.postal_fmt=tsc(u"%c%N%T%N%s %h %e %r%N%b%N%d%N%f%N%a%N"),.country_name=tsc(u"جۇڭگو"),.country_ab2=tsc(u"CN"),.country_ab3=tsc(u"CHN"),.country_num=156,.country_car=tsc(u"CHN"),.country_isbn=tsc(u"7"),.lang_name=tsc(u"ئۇيغۇرچە"),.lang_ab=tsc(u"ug"),.lang_term=tsc(u"uig"),.lang_lib=tsc(u"uig")},.measurement={.measurement=1}};

inline constexpr u32lc_all u32lc_all_global{.identification={.name=tsc(U"ug_CN"),.encoding=tsc(FAST_IO_LOCALE_UENCODING),.title=tsc(U"Uyghur locale for China"),.source=tsc(U"fast_io"),.address=tsc(U"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(U"Eagle Burkut\t\t;\t\tfast_io"),.email=tsc(U"eagle.burkut@gmail.com;euloanty@live.com"),.tel=tsc(U""),.fax=tsc(U""),.language=tsc(U"Uyghur"),.territory=tsc(U"China"),.revision=tsc(U"2.00"),.date=tsc(U"2011-02-26")},.monetary={.int_curr_symbol=tsc(U"CNY "),.currency_symbol=tsc(U"￥"),.mon_decimal_point=tsc(U"."),.mon_thousands_sep=tsc(U","),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(U""),.negative_sign=tsc(U"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.int_p_cs_precedes=1,.int_p_sep_by_space=0,.int_n_cs_precedes=1,.int_n_sep_by_space=0,.p_sign_posn=4,.n_sign_posn=4,.int_p_sign_posn=1,.int_n_sign_posn=1},.numeric={.decimal_point=tsc(U"."),.thousands_sep=tsc(U","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(U"ي"),tsc(U"د"),tsc(U"س"),tsc(U"چ"),tsc(U"پ"),tsc(U"ج"),tsc(U"ش")},.day={tsc(U"يەكشەنبە"),tsc(U"دۈشەنبە"),tsc(U"سەيشەنبە"),tsc(U"چارشەنبە"),tsc(U"پەيشەنبە"),tsc(U"جۈمە"),tsc(U"شەنبە")},.abmon={tsc(U"يانۋار"),tsc(U"فېۋرال"),tsc(U"مارت"),tsc(U"ئاپرېل"),tsc(U"ماي"),tsc(U"ئىيۇن"),tsc(U"ئىيۇل"),tsc(U"ئاۋغۇست"),tsc(U"سېنتەبىر"),tsc(U"ئۆكتەبىر"),tsc(U"نويابىر"),tsc(U"دېكابىر")},.mon={tsc(U"يانۋار"),tsc(U"فېۋرال"),tsc(U"مارت"),tsc(U"ئاپرېل"),tsc(U"ماي"),tsc(U"ئىيۇن"),tsc(U"ئىيۇل"),tsc(U"ئاۋغۇست"),tsc(U"سېنتەبىر"),tsc(U"ئۆكتەبىر"),tsc(U"نويابىر"),tsc(U"دېكابىر")},.d_t_fmt=tsc(U"%a، %d-%m-%Y، %T"),.d_fmt=tsc(U"%a، %d-%m-%Y"),.t_fmt=tsc(U"%T"),.date_fmt=tsc(U"%a، %d-%m-%Y، %T"),.am_pm={tsc(U""),tsc(U"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(U"^[+1yYھ]"),.noexpr=tsc(U"^[-0nNي]"),.yesstr=tsc(U"ھەئە"),.nostr=tsc(U"ياق")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(U"+%c %a %l"),.tel_dom_fmt=tsc(U"0%a %l"),.int_select=tsc(U"00"),.int_prefix=tsc(U"86")},.name={.name_fmt=tsc(U"%f%t%g%t%d"),.name_gen=tsc(U""),.name_miss=tsc(U"小姐"),.name_mr=tsc(U"先生"),.name_mrs=tsc(U"太太"),.name_ms=tsc(U"女士")},.address={.postal_fmt=tsc(U"%c%N%T%N%s %h %e %r%N%b%N%d%N%f%N%a%N"),.country_name=tsc(U"جۇڭگو"),.country_ab2=tsc(U"CN"),.country_ab3=tsc(U"CHN"),.country_num=156,.country_car=tsc(U"CHN"),.country_isbn=tsc(U"7"),.lang_name=tsc(U"ئۇيغۇرچە"),.lang_ab=tsc(U"ug"),.lang_term=tsc(U"uig"),.lang_lib=tsc(U"uig")},.measurement={.measurement=1}};


}
}

#include"../main.h"