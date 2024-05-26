﻿#include"../localedef.h"

namespace fast_io_i18n
{
namespace
{

inline constexpr lc_all lc_all_global{.monetary={.int_curr_symbol=tsc(""),.currency_symbol=tsc(""),.mon_decimal_point=tsc("."),.mon_thousands_sep=tsc(""),.positive_sign=tsc(""),.negative_sign=tsc(""),.int_frac_digits=SIZE_MAX,.frac_digits=SIZE_MAX,.p_cs_precedes=SIZE_MAX,.p_sep_by_space=SIZE_MAX,.n_cs_precedes=SIZE_MAX,.n_sep_by_space=SIZE_MAX,.p_sign_posn=SIZE_MAX,.n_sign_posn=SIZE_MAX},.numeric={.decimal_point=tsc("."),.thousands_sep=tsc("")},.time={.abday={tsc("Sun"),tsc("Mon"),tsc("Tue"),tsc("Wed"),tsc("Thu"),tsc("Fri"),tsc("Sat")},.day={tsc("Sunday"),tsc("Monday"),tsc("Tuesday"),tsc("Wednesday"),tsc("Thursday"),tsc("Friday"),tsc("Saturday")},.abmon={tsc("Jan"),tsc("Feb"),tsc("Mar"),tsc("Apr"),tsc("May"),tsc("Jun"),tsc("Jul"),tsc("Aug"),tsc("Sep"),tsc("Oct"),tsc("Nov"),tsc("Dec")},.mon={tsc("January"),tsc("February"),tsc("March"),tsc("April"),tsc("May"),tsc("June"),tsc("July"),tsc("August"),tsc("September"),tsc("October"),tsc("November"),tsc("December")},.d_t_fmt=tsc("%a %b %e %H:%M:%S %Y"),.d_fmt=tsc("%m/%d/%y"),.t_fmt=tsc("%H:%M:%S"),.t_fmt_ampm=tsc("%I:%M:%S %p"),.date_fmt=tsc("%a %b %e %H:%M:%S %Z %Y"),.am_pm={tsc("AM"),tsc("PM")}},.messages={.yesexpr=tsc("^[yY]"),.noexpr=tsc("^[nN]"),.yesstr=tsc("Yes"),.nostr=tsc("No")}};

inline constexpr wlc_all wlc_all_global{.monetary={.int_curr_symbol=tsc(L""),.currency_symbol=tsc(L""),.mon_decimal_point=tsc(L"."),.mon_thousands_sep=tsc(L""),.positive_sign=tsc(L""),.negative_sign=tsc(L""),.int_frac_digits=SIZE_MAX,.frac_digits=SIZE_MAX,.p_cs_precedes=SIZE_MAX,.p_sep_by_space=SIZE_MAX,.n_cs_precedes=SIZE_MAX,.n_sep_by_space=SIZE_MAX,.p_sign_posn=SIZE_MAX,.n_sign_posn=SIZE_MAX},.numeric={.decimal_point=tsc(L"."),.thousands_sep=tsc(L"")},.time={.abday={tsc(L"Sun"),tsc(L"Mon"),tsc(L"Tue"),tsc(L"Wed"),tsc(L"Thu"),tsc(L"Fri"),tsc(L"Sat")},.day={tsc(L"Sunday"),tsc(L"Monday"),tsc(L"Tuesday"),tsc(L"Wednesday"),tsc(L"Thursday"),tsc(L"Friday"),tsc(L"Saturday")},.abmon={tsc(L"Jan"),tsc(L"Feb"),tsc(L"Mar"),tsc(L"Apr"),tsc(L"May"),tsc(L"Jun"),tsc(L"Jul"),tsc(L"Aug"),tsc(L"Sep"),tsc(L"Oct"),tsc(L"Nov"),tsc(L"Dec")},.mon={tsc(L"January"),tsc(L"February"),tsc(L"March"),tsc(L"April"),tsc(L"May"),tsc(L"June"),tsc(L"July"),tsc(L"August"),tsc(L"September"),tsc(L"October"),tsc(L"November"),tsc(L"December")},.d_t_fmt=tsc(L"%a %b %e %H:%M:%S %Y"),.d_fmt=tsc(L"%m/%d/%y"),.t_fmt=tsc(L"%H:%M:%S"),.t_fmt_ampm=tsc(L"%I:%M:%S %p"),.date_fmt=tsc(L"%a %b %e %H:%M:%S %Z %Y"),.am_pm={tsc(L"AM"),tsc(L"PM")}},.messages={.yesexpr=tsc(L"^[yY]"),.noexpr=tsc(L"^[nN]"),.yesstr=tsc(L"Yes"),.nostr=tsc(L"No")}};

inline constexpr u8lc_all u8lc_all_global{.monetary={.int_curr_symbol=tsc(u8""),.currency_symbol=tsc(u8""),.mon_decimal_point=tsc(u8"."),.mon_thousands_sep=tsc(u8""),.positive_sign=tsc(u8""),.negative_sign=tsc(u8""),.int_frac_digits=SIZE_MAX,.frac_digits=SIZE_MAX,.p_cs_precedes=SIZE_MAX,.p_sep_by_space=SIZE_MAX,.n_cs_precedes=SIZE_MAX,.n_sep_by_space=SIZE_MAX,.p_sign_posn=SIZE_MAX,.n_sign_posn=SIZE_MAX},.numeric={.decimal_point=tsc(u8"."),.thousands_sep=tsc(u8"")},.time={.abday={tsc(u8"Sun"),tsc(u8"Mon"),tsc(u8"Tue"),tsc(u8"Wed"),tsc(u8"Thu"),tsc(u8"Fri"),tsc(u8"Sat")},.day={tsc(u8"Sunday"),tsc(u8"Monday"),tsc(u8"Tuesday"),tsc(u8"Wednesday"),tsc(u8"Thursday"),tsc(u8"Friday"),tsc(u8"Saturday")},.abmon={tsc(u8"Jan"),tsc(u8"Feb"),tsc(u8"Mar"),tsc(u8"Apr"),tsc(u8"May"),tsc(u8"Jun"),tsc(u8"Jul"),tsc(u8"Aug"),tsc(u8"Sep"),tsc(u8"Oct"),tsc(u8"Nov"),tsc(u8"Dec")},.mon={tsc(u8"January"),tsc(u8"February"),tsc(u8"March"),tsc(u8"April"),tsc(u8"May"),tsc(u8"June"),tsc(u8"July"),tsc(u8"August"),tsc(u8"September"),tsc(u8"October"),tsc(u8"November"),tsc(u8"December")},.d_t_fmt=tsc(u8"%a %b %e %H:%M:%S %Y"),.d_fmt=tsc(u8"%m/%d/%y"),.t_fmt=tsc(u8"%H:%M:%S"),.t_fmt_ampm=tsc(u8"%I:%M:%S %p"),.date_fmt=tsc(u8"%a %b %e %H:%M:%S %Z %Y"),.am_pm={tsc(u8"AM"),tsc(u8"PM")}},.messages={.yesexpr=tsc(u8"^[yY]"),.noexpr=tsc(u8"^[nN]"),.yesstr=tsc(u8"Yes"),.nostr=tsc(u8"No")}};

inline constexpr u16lc_all u16lc_all_global{.monetary={.int_curr_symbol=tsc(u""),.currency_symbol=tsc(u""),.mon_decimal_point=tsc(u"."),.mon_thousands_sep=tsc(u""),.positive_sign=tsc(u""),.negative_sign=tsc(u""),.int_frac_digits=SIZE_MAX,.frac_digits=SIZE_MAX,.p_cs_precedes=SIZE_MAX,.p_sep_by_space=SIZE_MAX,.n_cs_precedes=SIZE_MAX,.n_sep_by_space=SIZE_MAX,.p_sign_posn=SIZE_MAX,.n_sign_posn=SIZE_MAX},.numeric={.decimal_point=tsc(u"."),.thousands_sep=tsc(u"")},.time={.abday={tsc(u"Sun"),tsc(u"Mon"),tsc(u"Tue"),tsc(u"Wed"),tsc(u"Thu"),tsc(u"Fri"),tsc(u"Sat")},.day={tsc(u"Sunday"),tsc(u"Monday"),tsc(u"Tuesday"),tsc(u"Wednesday"),tsc(u"Thursday"),tsc(u"Friday"),tsc(u"Saturday")},.abmon={tsc(u"Jan"),tsc(u"Feb"),tsc(u"Mar"),tsc(u"Apr"),tsc(u"May"),tsc(u"Jun"),tsc(u"Jul"),tsc(u"Aug"),tsc(u"Sep"),tsc(u"Oct"),tsc(u"Nov"),tsc(u"Dec")},.mon={tsc(u"January"),tsc(u"February"),tsc(u"March"),tsc(u"April"),tsc(u"May"),tsc(u"June"),tsc(u"July"),tsc(u"August"),tsc(u"September"),tsc(u"October"),tsc(u"November"),tsc(u"December")},.d_t_fmt=tsc(u"%a %b %e %H:%M:%S %Y"),.d_fmt=tsc(u"%m/%d/%y"),.t_fmt=tsc(u"%H:%M:%S"),.t_fmt_ampm=tsc(u"%I:%M:%S %p"),.date_fmt=tsc(u"%a %b %e %H:%M:%S %Z %Y"),.am_pm={tsc(u"AM"),tsc(u"PM")}},.messages={.yesexpr=tsc(u"^[yY]"),.noexpr=tsc(u"^[nN]"),.yesstr=tsc(u"Yes"),.nostr=tsc(u"No")}};

inline constexpr u32lc_all u32lc_all_global{.monetary={.int_curr_symbol=tsc(U""),.currency_symbol=tsc(U""),.mon_decimal_point=tsc(U"."),.mon_thousands_sep=tsc(U""),.positive_sign=tsc(U""),.negative_sign=tsc(U""),.int_frac_digits=SIZE_MAX,.frac_digits=SIZE_MAX,.p_cs_precedes=SIZE_MAX,.p_sep_by_space=SIZE_MAX,.n_cs_precedes=SIZE_MAX,.n_sep_by_space=SIZE_MAX,.p_sign_posn=SIZE_MAX,.n_sign_posn=SIZE_MAX},.numeric={.decimal_point=tsc(U"."),.thousands_sep=tsc(U"")},.time={.abday={tsc(U"Sun"),tsc(U"Mon"),tsc(U"Tue"),tsc(U"Wed"),tsc(U"Thu"),tsc(U"Fri"),tsc(U"Sat")},.day={tsc(U"Sunday"),tsc(U"Monday"),tsc(U"Tuesday"),tsc(U"Wednesday"),tsc(U"Thursday"),tsc(U"Friday"),tsc(U"Saturday")},.abmon={tsc(U"Jan"),tsc(U"Feb"),tsc(U"Mar"),tsc(U"Apr"),tsc(U"May"),tsc(U"Jun"),tsc(U"Jul"),tsc(U"Aug"),tsc(U"Sep"),tsc(U"Oct"),tsc(U"Nov"),tsc(U"Dec")},.mon={tsc(U"January"),tsc(U"February"),tsc(U"March"),tsc(U"April"),tsc(U"May"),tsc(U"June"),tsc(U"July"),tsc(U"August"),tsc(U"September"),tsc(U"October"),tsc(U"November"),tsc(U"December")},.d_t_fmt=tsc(U"%a %b %e %H:%M:%S %Y"),.d_fmt=tsc(U"%m/%d/%y"),.t_fmt=tsc(U"%H:%M:%S"),.t_fmt_ampm=tsc(U"%I:%M:%S %p"),.date_fmt=tsc(U"%a %b %e %H:%M:%S %Z %Y"),.am_pm={tsc(U"AM"),tsc(U"PM")}},.messages={.yesexpr=tsc(U"^[yY]"),.noexpr=tsc(U"^[nN]"),.yesstr=tsc(U"Yes"),.nostr=tsc(U"No")}};


}
}

#include"../main.h"