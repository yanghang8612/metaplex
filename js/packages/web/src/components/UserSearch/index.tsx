import { shortenAddress } from '@oyster/common';
import { Select, Spin } from 'antd';
import { SelectProps } from 'antd/es/select';
import debounce from 'lodash/debounce';
import React, { useMemo, useRef, useState } from 'react';
import { useMeta } from '../../contexts';
import './styles.less';

export interface DebounceSelectProps<ValueType = any>
  extends Omit<SelectProps<ValueType>, 'options' | 'children'> {
  fetchOptions: (search: string) => Promise<ValueType[]>;
  debounceTimeout?: number;
}

function DebounceSelect<
  ValueType extends {
    key?: string;
    label: React.ReactNode;
    value: string | number;
  } = any,
>({ fetchOptions, debounceTimeout = 800, ...props }: DebounceSelectProps) {
  const [fetching, setFetching] = useState(false);
  const [options, setOptions] = useState<ValueType[]>([]);
  const fetchRef = useRef(0);

  const debounceFetcher = useMemo(() => {
    const loadOptions = (value: string) => {
      fetchRef.current += 1;
      const fetchId = fetchRef.current;
      setOptions([]);
      setFetching(true);

      fetchOptions(value).then(newOptions => {
        if (fetchId !== fetchRef.current) {
          // for fetch callback order
          return;
        }

        setOptions(newOptions);
        setFetching(false);
      });
    };

    return debounce(loadOptions, debounceTimeout);
  }, [fetchOptions, debounceTimeout]);

  return (
    <Select<ValueType>
      labelInValue
      filterOption={false}
      onSearch={debounceFetcher}
      notFoundContent={fetching ? <Spin size="small" /> : null}
      {...props}
      options={options}
    />
  );
}

// Usage of DebounceSelect
export interface UserValue {
  key: string;
  label: string;
  value: string;
}

export const UserSearch = (props: { setCreators: Function }) => {
  const [value, setValue] = React.useState<UserValue[]>([]);
  const { whitelistedCreatorsByCreator } = useMeta();

  return (
    <DebounceSelect
      className="user-selector"
      mode="multiple"
      size="large"
      value={value}
      placeholder="Select creator"
      fetchOptions={async (search: string) =>
        {
          const items = Object.values(whitelistedCreatorsByCreator)
          .filter(c => c.info.activated)
          .map(a => ({
            label: a.info.name || shortenAddress(a.info.address.toBase58()),
            value: a.info.address.toBase58()
          }));

          return items;
        }
      }
      onChange={newValue => {
        props.setCreators(newValue);
        setValue(newValue);
      }}
      style={{ width: '100%' }}
    />
  );
};
